use api_ict::route;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(route::config)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
