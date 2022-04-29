use actix_web::{App, HttpServer};
use backend::config_services;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    if let Err(e) = dotenv() {
        log::error!("Failed to load .env file: {:?}", e);
    }

    HttpServer::new(|| App::new().configure(config_services))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await?;
    Ok(())
}
