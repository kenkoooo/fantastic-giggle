use crate::Result;
use actix_web::{
    get,
    http::header::LOCATION,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};
use egg_mode::{
    auth::{access_token, authorize_url, request_token, verify_tokens},
    KeyPair, Token,
};
use fantastic_giggle_sql::{AccessToken, PgPool};
use serde::{Deserialize, Serialize};

pub fn config_services(cfg: &mut ServiceConfig) {
    cfg.service(login).service(callback).service(user);
}

#[get("/api/login")]
async fn login(consumer: web::Data<KeyPair>) -> Result<HttpResponse> {
    let consumer = consumer.as_ref().clone();
    let request_token = request_token(&consumer, "http://localhost:8080/api/callback").await?;
    let auth_url = authorize_url(&request_token);
    Ok(HttpResponse::Found()
        .append_header((LOCATION, auth_url))
        .finish())
}

#[derive(Deserialize)]
struct CallbackQuery {
    oauth_token: String,
    oauth_verifier: String,
}

#[get("/api/callback")]
async fn callback(
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
        AccessToken::save(
            pool.as_ref(),
            AccessToken {
                id: user_id as i64,
                access_key: access.key.to_string(),
                access_secret: access.secret.to_string(),
            },
        )
        .await?;
    }
    Ok(response.finish())
}

#[derive(Serialize)]
struct UserResponse {
    screen_name: String,
    id: u64,
}

#[get("/api/user")]
async fn user(request: HttpRequest, consumer: web::Data<KeyPair>) -> Result<HttpResponse> {
    let access = match request.token() {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::BadRequest().finish());
        }
    };
    let consumer = consumer.as_ref().clone();

    let token = Token::Access { consumer, access };
    let user = verify_tokens(&token).await?;
    Ok(HttpResponse::Ok().json(UserResponse {
        screen_name: user.screen_name.to_string(),
        id: user.id,
    }))
}

trait HttpRequestExt {
    fn token(&self) -> Option<KeyPair>;
}

impl HttpRequestExt for HttpRequest {
    fn token(&self) -> Option<KeyPair> {
        let key = self.cookie("key")?.value().to_string();
        let secret = self.cookie("secret")?.value().to_string();
        Some(KeyPair::new(key, secret))
    }
}
