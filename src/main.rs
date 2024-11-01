use actix_web::{App, HttpServer};
use api_ict::route;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap_or(8000);

    let server = HttpServer::new(|| App::new().configure(route::config)).bind(("0.0.0.0", port))?;

    println!("Server is running at port: {}", port);

    server.run().await
}
