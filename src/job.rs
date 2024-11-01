use serde_json::{from_str, json, Value};
use std::env;

const JOBS: &str = r#"
[
    {
        "id": "69201",
        "api_id": "3b900e4d-1667-ed11-9562-000d3a83015d",
        "duration": "4",
        "name": {
            "fr": "Développeuse/Développeur de business numérique CFC (dès 2023)",
            "de": "Entwickler/in digitales Business EFZ (ab 2023)",
            "it": "Sviluppatrice/Sviluppatore business digitale AFC (dal 2023)"
        }
    },
    {
        "id": "88601",
        "api_id": "4f50e7f3-6b82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne CFC développement d'applications",
            "de": "Informatiker/in EFZ Applikationsentwicklung",
            "it": "Informatico/a AFC Sviluppo di applicazioni"
        }
    },
    {
        "id": "88613",
        "api_id": "13d8d40b-6d82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne CFC Développement d'applications (dès 2021)",
            "de": "Informatiker/in EFZ Applikationsentwicklung (ab 2021)",
            "it": "Informatico/a AFC Sviluppo di applicazioni (dal 2021)"
        }
    },
    {
        "id": "88612",
        "api_id": "1eac87d6-6d82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne CFC exploitation et infrastructure (dès 2021)",
            "de": "Informatiker/in EFZ Plattformentwicklung (ab 2021)",
            "it": "Informatico/a AFC Gestione di infrastrutture (dal 2021)"
        }
    },
    {
        "id": "88614",
        "api_id": "03a95323-bf92-eb11-b1ac-000d3a831ef4",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne CFC informatique d'entreprise",
            "de": "Informatiker/in EFZ Betriebsinformatik",
            "it": "Informatico/a AFC Informatica aziendale"
        }
    },
    {
        "id": "88614",
        "api_id": "706fb04c-6e82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne CFC Informatique d'entreprise (dès 2021)",
            "de": "Betriebsinformatiker/in EFZ (ab 2021)",
            "it": "Informatico/a AFC Informatica aziendale (dal 2021)"
        }
    },
    {
        "id": "88603",
        "api_id": "56567396-6e82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne CFC Technique des systèmes",
            "de": "Informatiker/in EFZ Systemtechnik",
            "it": "Informatico/a AFC Tecnica dei sistemi"
        }
    },
    {
        "id": "88609",
        "api_id": "d1aa2e12-e592-eb11-b1ac-000d3a831ef4",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne du bâtiment CFC automatisation des bâtiments (dès 2021)",
            "de": "Gebäudeinformatiker/in EFZ Gebäudeautomation (ab 2021)",
            "it": "Informatico/a degli edifici domotica (dal 2021)"
        }
    },
    {
        "id": "88610",
        "api_id": "2459e01c-e592-eb11-b1ac-000d3a831ef4",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne du bâtiment CFC communication et multimédia (dès 2021)",
            "de": "Gebäudeinformatiker/in EFZ Kommunikation und Multimedia (ab 2021)",
            "it": "Informatico/a degli edifici comunicazione e multimedia (dal 2021)"
        }
    },
    {
        "id": "88608",
        "api_id": "a17098f7-6f82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Informaticien/ne du bâtiment CFC planification (dès 2021)",
            "de": "Gebäudeinformatiker/in EFZ Planung (ab 2021)",
            "it": "Informatico/a degli edifici AFC progettazione (dal 2021)"
        }
    },
    {
        "id": "88606",
        "api_id": "f2cb37d4-6e82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Médiamaticien/ne CFC (dès 2019)",
            "de": "Mediamatiker/in EFZ (ab 2019)",
            "it": "Mediamatico/a AFC (dal 2019)"
        }
    },
    {
        "id": "88605",
        "api_id": "f1e7a970-6f82-eb11-a812-0022486f6f83",
        "duration": "4",
        "name": {
            "fr": "Opératrice en informatique/Opérateur en informatique CFC",
            "de": "ICT-Fachfrau/ICT-Fachmann EFZ",
            "it": "Operatrice informatico/Operatore informatico AFC"
        }
    }
]    
"#;

pub async fn get_jobs(lang: &Option<String>) -> Result<Value, Box<dyn std::error::Error>> {
    let default_language = env::var("DEFAULT_LANGUAGE").unwrap_or_else(|_| "de".to_string());
    let language = lang.as_deref().unwrap_or(&default_language);

    let jobs: Vec<Value> = from_str(JOBS)?;

    let jobs_translated: Vec<Value> = jobs
        .iter()
        .map(|job| {
            json!({
                "id": job["id"],
                "name": job["name"].get(language).unwrap_or(&json!("")),
            })
        })
        .collect();

    Ok(json!(jobs_translated))
}

pub async fn get_api_id(job_id: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if job_id.is_empty() {
        return Ok(None);
    }

    let jobs: Vec<Value> = from_str(JOBS)?;

    let job = jobs
        .iter()
        .find(|job| job["id"] == job_id)
        .ok_or("Job not found")?;

    Ok(job["api_id"].as_str().map(|s| s.to_string()))
}
