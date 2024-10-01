use crate::auth;
use regex::Regex;
use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    value: Vec<Value>,
}

pub async fn get_modules(
    lang: &Option<String>,
    year: &Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let token = auth::get_token().await?;
    let client = reqwest::Client::new();
    let res = client
        .get("https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_modulmappings?$filter=beembk_Abschluss/beembk_abschlussid%20eq%20%2713d8d40b-6d82-eb11-a812-0022486f6f83%27&$expand=beembk_Lernort,beembk_Modul,beembk_Modultyp,beembk_Level")
        .bearer_auth(token)
        .send()
        .await?;

    let api_response: ApiResponse = res.json().await?;
    let re = Regex::new(r"^\d+").unwrap();
    let lang = lang.as_deref().unwrap_or("de");

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

            let (type_key, title_key, description_key) = match lang {
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

pub async fn get_module(id: &str, lang: &Option<String>) -> Result<Value, Box<dyn Error>> {
    let token = auth::get_token().await?;
    let client = reqwest::Client::new();

    let url = format!(
        "https://ictbb.crm17.dynamics.com/api/data/v9.1/beembk_handlungsnotwendigeskenntnises?$filter=startswith(beembk_hanoknr,'{}')",
        id
    );

    let res = client.get(&url).bearer_auth(token).send().await?;

    // Deserialize the response
    let api_response: ApiResponse = res.json().await?;

    // Check if the module exists
    if api_response.value.is_empty() {
        return Err("Module not found".into());
    }

    let module = &api_response.value[0]; // Get the first module

    let re = Regex::new(r"^\d+").unwrap();
    let lang = lang.as_deref().unwrap_or("de");

    let level_name = module["beembk_levelname"].as_str().unwrap_or("");

    let module_year = re
        .find(level_name)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or_default();

    let (type_key, title_key, description_key) = match lang {
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
    let r#type = module[type_key].as_str().unwrap_or("").to_string();
    let description = module[description_key].as_str().unwrap_or("").to_string();

    Ok(json!({
        "number": number,
        "description": description,
        "name": name,
        "year": module_year,
        "version": version,
        "last_modified": last_modified,
        "creation_date": creation_date,
        "type": r#type,
    }))
}
