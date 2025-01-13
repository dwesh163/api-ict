use crate::{auth, job};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_DURATION: Duration = Duration::from_secs(20736000); // 8 months in seconds

#[derive(Debug, Deserialize)]
struct ApiResponse {
    value: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    data: Value,
    timestamp: u64,
}

fn get_cache_path(cache_key: &str) -> String {
    format!(".cache/{}.json", cache_key)
}

fn is_cache_valid(timestamp: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now - timestamp < CACHE_DURATION.as_secs()
}

async fn get_cached_data<F, Fut>(
    cache_key: &str,
    fetch_data: F,
) -> Result<Value, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Value, Box<dyn std::error::Error>>>,
{
    if env::var("DISABLE_CACHE").unwrap_or_else(|_| "false".to_string()) == "true" {
        return fetch_data().await;
    }

    let cache_path = get_cache_path(cache_key);

    if Path::new(&cache_path).exists() {
        let cache_content = fs::read_to_string(&cache_path)?;
        let cache_entry: CacheEntry = serde_json::from_str(&cache_content)?;

        if is_cache_valid(cache_entry.timestamp) {
            return Ok(cache_entry.data);
        }
    }

    let fresh_data = fetch_data().await?;

    fs::create_dir_all(".cache")?;

    let cache_entry = CacheEntry {
        data: fresh_data.clone(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    fs::write(&cache_path, serde_json::to_string(&cache_entry)?)?;

    Ok(fresh_data)
}

pub async fn get_modules(
    lang: &Option<String>,
    year: &Option<String>,
    job_id: &Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let default_language = env::var("DEFAULT_LANGUAGE").unwrap_or_else(|_| "de".to_string());
    let language = lang.as_deref().unwrap_or(&default_language);

    let cache_key = format!(
        "modules_{}_{}_{}",
        language,
        year.as_deref().unwrap_or("default"),
        job_id.as_deref().unwrap_or("default")
    );

    get_cached_data(&cache_key, || async {
        let token = auth::get_token().await?;
        let client = reqwest::Client::new();
        let api_id = job::get_api_id(job_id.as_deref().unwrap_or("")).await?;
        let url = match api_id {
            Some(id) => format!(
                "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_modulmappings?$filter=beembk_Abschluss/beembk_abschlussid%20eq%20%27{}%27&$expand=beembk_Lernort,beembk_Modul,beembk_Modultyp,beembk_Level",
                id
            ),
            None => String::from(
                "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_modulmappings?$expand=beembk_Lernort,beembk_Modul,beembk_Modultyp,beembk_Level"
            ),
        };

        let res = client.get(url).bearer_auth(token).send().await?;
        let api_response: ApiResponse = res.json().await?;
        let re = Regex::new(r"^\d+").unwrap();
        

        use std::collections::HashMap;
        let mut modules_by_number: HashMap<i64, Vec<&Value>> = HashMap::new();

        for module in api_response.value.iter() {
            if let Some(number) = module["beembk_Modul"]["beembk_modulnummer"]
                .as_str()
                .and_then(|n| n.parse::<i64>().ok())
            {
                modules_by_number.entry(number).or_default().push(module);
            }
        }

        let filtered_modules: Vec<Value> = modules_by_number
            .into_iter()
            .filter_map(|(_, mut modules)| {
                modules.sort_by(|a, b| {
                    let version_a = a["beembk_Modul"]["versionnumber"].as_f64().unwrap_or(0.0);
                    let version_b = b["beembk_Modul"]["versionnumber"].as_f64().unwrap_or(0.0);
                    version_b
                        .partial_cmp(&version_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let module = modules.first()?;

                let level_name = module["beembk_Level"]["beembk_levelname"]
                    .as_str()
                    .unwrap_or("");
                let module_year = re
                    .find(level_name)
                    .and_then(|m| m.as_str().parse::<i64>().ok())
                    .unwrap_or_default();

                if let Some(year) = year {
                    if module_year != year.parse::<i64>().unwrap_or_default() {
                        return None;
                    }
                }

                let (type_key, title_key, description_key) = match language {
                    "de" => (
                        "beembk_lernortname",
                        "beembk_modultitel",
                        "beembk_objektbeschreibung",
                    ),
                    "fr" => (
                        "beembk_lernortname_fr",
                        "beembk_modultitel_fr",
                        "beembk_objektbeschreibung_fr",
                    ),
                    "it" => (
                        "beembk_lernortname_it",
                        "beembk_modultitel_it",
                        "beembk_objektbeschreibung_it",
                    ),
                    _ => return None,
                };

                let number = module["beembk_Modul"]["beembk_modulnummer"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<i64>()
                    .unwrap_or_default();
                let name = module["beembk_Modul"][title_key].as_str().unwrap_or("");
                let version = module["beembk_Modul"]["beembk_version"]
                    .as_i64()
                    .unwrap_or_default();
                let last_modified = module["beembk_Modul"]["modifiedon"].as_str().unwrap_or("");
                let creation_date = module["beembk_Modul"]["createdon"].as_str().unwrap_or("");
                let r#type = module["beembk_Lernort"][type_key].as_str().unwrap_or("");
                let description = module["beembk_Modul"][description_key]
                    .as_str()
                    .unwrap_or("");

                Some(json!({
                    "number": number,
                    "description": description,
                    "name": name,
                    "year": module_year,
                    "version": version,
                    "last_modified": last_modified,
                    "creation_date": creation_date,
                    "type": r#type,
                }))
            })
            .collect();

        Ok(json!(filtered_modules))
    }).await
}

pub async fn get_module(
    id: &str,
    lang: &Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let default_language = env::var("DEFAULT_LANGUAGE").unwrap_or_else(|_| "de".to_string());
    let language = lang.as_deref().unwrap_or(&default_language);
    let cache_key = format!("module_{}_{}", id, language);

    get_cached_data(&cache_key, || async {
        let token = auth::get_token().await?;
        let client = reqwest::Client::new();

        let url = format!(
            "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_moduls?$filter=contains(beembk_modulnummer,'{}')",
            id
        );

        let res = client.get(&url).bearer_auth(token).send().await?;
        let api_response: ApiResponse = res.json().await?;

        if api_response.value.is_empty() {
            return Err("Module not found".into());
        }

        let module = api_response
            .value
            .iter()
            .max_by_key(|module| module["versionnumber"].as_i64().unwrap_or(0))
            .ok_or("No modules found")?;

        let module_detail = get_module_detail(id, language).await?;
        let year = module_detail["year"].as_i64().unwrap_or_default();
        let r#type = module_detail["type"].as_str().unwrap_or("").to_string();

        let (title_key, description_key, competence_key, pdf_key) = match language {
            "de" => (
                "beembk_modultitel",
                "beembk_objektbeschreibung",
                "beembk_kompetenz",
                "beembk_pdfname_de",
            ),
            "fr" => (
                "beembk_modultitel_fr",
                "beembk_objektbeschreibung_fr",
                "beembk_kompetenz_fr",
                "beembk_pdfname_fr",
            ),
            "it" => (
                "beembk_modultitel_it",
                "beembk_objektbeschreibung_it",
                "beembk_kompetenz_it",
                "beembk_pdfname_it",
            ),
            _ => return Err("Unsupported language".into()),
        };

        let number = module["beembk_modulnummer"]
            .as_str()
            .unwrap_or("")
            .parse::<i64>()
            .unwrap_or_default();

        let name = module[title_key].as_str().unwrap_or("").to_string();
        let version = module["beembk_version"].as_i64().unwrap_or_default();
        let last_modified = module["modifiedon"].as_str().unwrap_or("").to_string();
        let creation_date = module["createdon"].as_str().unwrap_or("").to_string();
        let description = module[description_key].as_str().unwrap_or("").to_string();
        let competence = module[competence_key].as_str().unwrap_or("").to_string();
        let pdf = module[pdf_key].as_str().unwrap_or("").to_string();

        let objectives = get_module_objectives(id, language).await?;

        Ok(json!({
            "number": number,
            "description": description,
            "name": name,
            "year": year,
            "type": r#type,
            "version": version,
            "last_modified": last_modified,
            "creation_date": creation_date,
            "pdf": format!("https://www.modulbaukasten.ch/Module/{}", pdf),
            "competence": competence,
            "objectives": objectives,
        }))
    }).await
}

async fn get_module_detail(
    id: &str,
    lang: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let token = auth::get_token().await?;
    let client = reqwest::Client::new();

    let url = format!(
            "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_modulmappings?$filter=beembk_Modul/beembk_modulnummer%20eq%20'{}'&$expand=beembk_Lernort,beembk_Modul,beembk_Modultyp,beembk_Level",
            id
        );
    let res = client.get(&url).bearer_auth(token).send().await?;
    let api_response: ApiResponse = res.json().await?;

    if api_response.value.is_empty() {
        return Err("Module not found".into());
    }

    let module = api_response
        .value
        .iter()
        .max_by_key(|module| module["versionnumber"].as_i64().unwrap_or(0))
        .ok_or("No modules found")?;

    let re = Regex::new(r"^\d+").map_err(|e| format!("Invalid regex: {}", e))?;
    let level_name = module["beembk_Level"]["beembk_levelname"]
        .as_str()
        .unwrap_or("");
    let module_year = re
        .find(level_name)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or_default();

    let type_key = match lang {
        "de" => "beembk_lernortname",
        "fr" => "beembk_lernortname_fr",
        "it" => "beembk_lernortname_it",
        _ => return Err("Unsupported language".into()),
    };

    let r#type = module["beembk_Lernort"][type_key].as_str().unwrap_or("");

    Ok(json!({
        "year": module_year,
        "type": r#type,
    }))
}

pub async fn get_module_objectives(
    id: &str,
    lang: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let token = auth::get_token().await?;
    let client = reqwest::Client::new();

    let url = format!(
        "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_handlungsziels?$filter=startswith(beembk_handlungszielnr,'{}')",
        id
    );
    let res = client.get(&url).bearer_auth(&token).send().await?;
    let api_response: ApiResponse = res.json().await?;

    let details_url = format!(
        "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_handlungsnotwendigeskenntnises?$filter=startswith(beembk_hanoknr,'{}')",
        id
    );
    let details_res = client.get(&details_url).bearer_auth(&token).send().await?;
    let details_api_response: ApiResponse = details_res.json().await?;

    let (detail_key, name_key) = match lang {
        "de" => ("beembk_hanok", "beembk_handlungsziel"),
        "fr" => ("beembk_hanok_fr", "beembk_handlungsziel_fr"),
        "it" => ("beembk_hanok_it", "beembk_handlungsziel_it"),
        _ => return Err("Unsupported language".into()),
    };

    let objectives: Vec<Value> = api_response
        .value
        .iter()
        .enumerate()
        .map(|(objectives_index, objective)| {
            let name = objective
                .get(name_key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let details: Vec<Value> = details_api_response
                .value
                .iter()
                .filter_map(|detail| {
                    let detail_nr = detail.get("beembk_hanoknr").and_then(|v| v.as_str())?;
                    if detail_nr.starts_with(&format!("{}.{}", id, objectives_index + 1)) {
                        let detail_name = detail
                            .get(detail_key)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(json!(detail_name + " " + detail_nr))
                    } else {
                        None
                    }
                })
                .collect();

            json!({
                "name": name,
                "details": details
            })
        })
        .collect();

    Ok(json!(objectives))
}

// async fn get_module_courses(id: &str, lang: &str) -> Result<Value, Box<dyn std::error::Error>> {
//     let token = auth::get_token().await?;
//     let client = reqwest::Client::new();

//     let url = format!(
//         "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_modulmappings?$filter=beembk_Modul/beembk_modulid%20eq%20%2714ac5969-0678-eb11-a812-000d3a831967%27&$expand=beembk_Abschluss,'{}')",
//         id
//     );

//     println!("{:?}", url);

//     let res = client.get(&url).bearer_auth(token).send().await?;
//     let api_response: ApiResponse = res.json().await?;

//     println!("{:?}", api_response);

//     Ok(json!(courses))
// }
