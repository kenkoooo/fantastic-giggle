use crate::Result;
use actix_web::{
    get,
    http::header::LOCATION,
    web::{self},
    HttpResponse,
};
use egg_mode::{
    auth::{access_token, authorize_url, request_token},
    KeyPair, Token,
};
use fantastic_giggle_sql::{PgPool, User};
use serde::Deserialize;

#[get("/api/login")]
pub(crate) async fn login(consumer: web::Data<KeyPair>) -> Result<HttpResponse> {
    let consumer = consumer.as_ref().clone();
    let request_token = request_token(&consumer, "http://localhost:8080/api/callback").await?;
    let auth_url = authorize_url(&request_token);
    Ok(HttpResponse::Found()
        .append_header((LOCATION, auth_url))
        .finish())
}

#[derive(Deserialize)]
pub(crate) struct CallbackQuery {
    oauth_token: String,
    oauth_verifier: String,
}

#[get("/api/callback")]
pub(crate) async fn callback(
    query: web::Query<CallbackQuery>,
    pool: web::Data<PgPool>,
    consumer: web::Data<KeyPair>,
) -> Result<HttpResponse> {
    let query = query.into_inner();
    let consumer = consumer.as_ref().clone();
    let request_token = KeyPair::new(query.oauth_token, "");
    let (token, user_id, _) = access_token(consumer, &request_token, &query.oauth_verifier).await?;

    let mut response = HttpResponse::Found();
    response.append_header((LOCATION, "/"));
    if let Token::Access {
        consumer: _,
        access,
    } = token
    {
        User::save(
            pool.as_ref(),
            User {
                id: user_id as i64,
                access_key: access.key.to_string(),
                access_secret: access.secret.to_string(),
            },
        )
        .await?;
    }
    Ok(response.finish())
}
