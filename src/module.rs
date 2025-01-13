use crate::{auth, job};
use regex::Regex;
use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    value: Vec<Value>,
}

pub async fn get_modules(
    lang: &Option<String>,
    year: &Option<String>,
    job_id: &Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
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
    let default_language = env::var("DEFAULT_LANGUAGE").unwrap_or_else(|_| "de".to_string());
    let language = lang.as_deref().unwrap_or(&default_language);

    let filtered_modules: Vec<Value> = api_response
        .value
        .iter()
        .filter_map(|module| {
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
}

pub async fn get_module(
    id: &str,
    lang: &Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
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

    let modules = &api_response.value.len();

    let module = &api_response.value[0];

    let default_language = env::var("DEFAULT_LANGUAGE").unwrap_or_else(|_| "de".to_string());
    let language = lang.as_deref().unwrap_or(&default_language);

    let module_detail = get_module_detail(id, lang).await?;
    let module_year = module_detail["year"].as_i64().unwrap_or_default();
    let module_type = module_detail["type"].as_str().unwrap_or("").to_string();

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
        "year": module_year,
        "type": module_type,
        "version": version,
        "last_modified": last_modified,
        "creation_date": creation_date,
        "pdf": format!("https://www.modulbaukasten.ch/Module/{}", pdf),
        "competence": competence,
        "objectives": objectives,
    }))
}

async fn get_module_detail(
    id: &str,
    lang: &Option<String>,
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

    let module = &api_response.value[0];
    let re = Regex::new(r"^\d+").map_err(|e| format!("Invalid regex: {}", e))?;

    let level_name = module["beembk_Level"]["beembk_levelname"]
        .as_str()
        .unwrap_or("");
    let module_year = re
        .find(level_name)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or_default();

    let type_key = match lang.as_deref().unwrap_or("de") {
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
