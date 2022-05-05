use actix_web::{web, App, HttpServer};
use egg_mode::KeyPair;
use fantastic_giggle_backend::config_services;
use fantastic_giggle_sql::PgPool;
use fantastic_giggle_worker::{Followers, Friends, IdSynchronizer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("SQL connection failure");

    let api_key = std::env::var("API_KEY").expect("API_KEY is not set");
    let api_secret = std::env::var("API_SECRET").expect("API_SECRET is not set");
    let consumer = KeyPair::new(api_key, api_secret);

    let pool1 = pool.clone();
    let consumer1 = consumer.clone();
    let followers = tokio::spawn(async move {
        let synchronizer = IdSynchronizer::new(consumer1, pool1.clone(), Followers { pool: pool1 });
        synchronizer.run().await;
    });

    let pool1 = pool.clone();
    let consumer1 = consumer.clone();
    let friends = tokio::spawn(async move {
        let synchronizer = IdSynchronizer::new(consumer1, pool1.clone(), Friends { pool: pool1 });
        synchronizer.run().await;
    });

    HttpServer::new(move || {
        let consumer = consumer.clone();
        let pool = pool.clone();
        App::new()
            .configure(config_services)
            .app_data(web::Data::new(consumer))
            .app_data(web::Data::new(pool))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;
    followers.await.unwrap();
    friends.await.unwrap();
    Ok(())
}
