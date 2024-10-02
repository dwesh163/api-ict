use actix_web::{App, HttpServer};
use api_ict::route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server =
        HttpServer::new(|| App::new().configure(route::config)).bind(("127.0.0.1", 8000))?;

    println!("Server is running at port : 8000");

    server.run().await
}
