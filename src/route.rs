use crate::module::get_modules;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct QueryParams {
    lang: Option<String>,
    year: Option<String>,
}

#[get("/")]
async fn index(query: web::Query<QueryParams>) -> impl Responder {
    let modules = match get_modules(&query.lang, &query.year).await {
        Ok(modules) => modules,
        Err(_) => json!({ "error": "Error fetching modules" }),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json(modules)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
