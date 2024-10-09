use crate::module::{get_module, get_modules};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct QueryParams {
    pub lang: Option<String>,
    pub year: Option<String>,
}

#[get("/")]
async fn index(query: web::Query<QueryParams>) -> impl Responder {
    let modules = match get_modules(&query.lang, &query.year).await {
        Ok(modules) => modules,
        Err(err) => {
            eprintln!("Error fetching module: {:?}", err);

            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .json(json!({ "error": "Error fetching module" }));
        }
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json(modules)
}

#[get("/module/{id}")]
async fn get_module_by_id(id: web::Path<String>, query: web::Query<QueryParams>) -> impl Responder {
    let module = match get_module(&id.into_inner(), &query.lang).await {
        Ok(module) => module,
        Err(err) => {
            if err.to_string() == "Module not found" {
                return HttpResponse::NotFound()
                    .content_type("application/json")
                    .json(json!({ "error": "Module not found" }));
            }

            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .json(json!({ "error": "Error fetching module" }));
        }
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json(module)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(index).service(get_module_by_id);
}
