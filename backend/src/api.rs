use crate::Result;
use actix_web::{
    cookie::Cookie,
    get,
    http::header::LOCATION,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};
use egg_mode::{
    auth::{access_token, authorize_url, request_token, verify_tokens},
    KeyPair, Token,
};
use serde::{Deserialize, Serialize};

pub fn config_services(cfg: &mut ServiceConfig) {
    cfg.service(login).service(callback).service(user);
}

#[get("/api/login")]
async fn login() -> Result<HttpResponse> {
    let token = load_token()?;
    let request_token = request_token(&token, "http://localhost:8080/api/callback").await?;
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
async fn callback(query: web::Query<CallbackQuery>) -> Result<HttpResponse> {
    let query = query.into_inner();
    let token = load_token()?;
    let request_token = KeyPair::new(query.oauth_token, "");
    let (token, _, _) = access_token(token, &request_token, &query.oauth_verifier).await?;

    let mut response = HttpResponse::Found();
    response.append_header((LOCATION, "http://localhost:8080/"));
    if let Token::Access {
        consumer: _,
        access,
    } = token
    {
        response.cookie(Cookie::new("key", access.key));
        response.cookie(Cookie::new("secret", access.secret));
    }
    Ok(response.finish())
}

#[derive(Serialize)]
struct UserResponse {
    screen_name: String,
    id: u64,
}

#[get("/api/user")]
async fn user(request: HttpRequest) -> Result<HttpResponse> {
    let access = match request.token() {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::BadRequest().finish());
        }
    };
    let consumer = load_token()?;

    let token = Token::Access { consumer, access };
    let user = verify_tokens(&token).await?;
    Ok(HttpResponse::Ok().json(UserResponse {
        screen_name: user.screen_name.to_string(),
        id: user.id,
    }))
}

fn load_token() -> Result<KeyPair> {
    let api_key = std::env::var("API_KEY")?;
    let api_secret = std::env::var("API_SECRET")?;
    Ok(KeyPair::new(api_key, api_secret))
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
