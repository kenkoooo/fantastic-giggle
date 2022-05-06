mod auth;

mod error;
pub(crate) use error::Result;

use actix_web::web::ServiceConfig;

pub fn config_services(cfg: &mut ServiceConfig) {
    cfg.service(auth::login).service(auth::callback);
}
