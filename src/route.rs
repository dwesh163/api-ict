use crate::documentation::get_documentation;
use crate::job::get_jobs;
use crate::module::{get_module, get_modules};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct QueryParams {
    pub lang: Option<String>,
    pub year: Option<String>,
    pub job_id: Option<String>,
}

#[get("/")]
async fn index() -> impl Responder {
    match get_documentation().await {
        Ok(documentation) => HttpResponse::Ok()
            .content_type("application/json")
            .json(documentation),
        Err(err) => {
            eprintln!("Error fetching documentation: {:?}", err);

            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .json(json!({ "error": "Error fetching documentation" }));
        }
    }
}

#[get("/jobs")]
async fn jobs(query: web::Query<QueryParams>) -> impl Responder {
    let jobs = match get_jobs(&query.lang).await {
        Ok(jobs) => jobs,
        Err(err) => {
            eprintln!("Error fetching jobs: {:?}", err);

            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .json(json!({ "error": "Error fetching jobs" }));
        }
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json(jobs)
}

#[get("/modules")]
async fn modules(query: web::Query<QueryParams>) -> impl Responder {
    let modules = match get_modules(&query.lang, &query.year, &query.job_id).await {
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
async fn module_by_id(id: web::Path<String>, query: web::Query<QueryParams>) -> impl Responder {
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
    cfg.service(index)
        .service(module_by_id)
        .service(modules)
        .service(jobs);
}
