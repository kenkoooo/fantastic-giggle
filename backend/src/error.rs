use std::fmt::Display;

pub type Result<T> = std::result::Result<T, ActixError>;

#[derive(Debug)]
pub struct ActixError {
    cause: Box<dyn std::error::Error>,
}

impl Display for ActixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cause)
    }
}
impl actix_web::error::ResponseError for ActixError {}

impl<E: 'static + std::error::Error> From<E> for ActixError {
    fn from(error: E) -> Self {
        Self {
            cause: Box::new(error),
        }
    }
}
