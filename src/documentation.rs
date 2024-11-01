use serde_json::{json, Value};

pub async fn get_documentation() -> Result<Value, Box<dyn std::error::Error>> {
    let documentation = json!({
        "description": "This API allows you to list all ICT modules.",
        "endpoints": [
            {
                "url": "/",
                "method": "GET",
            },
            {
                "url": "/jobs",
                "method": "GET",
            },
            {
                "url": "/modules",
                "method": "GET",
                "parameters": [
                    "job_id",
                    "lang" ,
                    "year"
                ]
            },
            {
                "url": "/modules/{moduleId}",
                "method": "GET",
                "parameters": [
                    "lang",
                    "year"
                ],
            }
        ]
    });

    Ok(documentation)
}
