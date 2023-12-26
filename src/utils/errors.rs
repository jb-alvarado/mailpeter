use std::io;

use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use toml;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "BadRequest: {_0}")]
    BadRequest(String),

    #[display(fmt = "Conflict: {_0}")]
    Conflict(String),

    #[display(fmt = "NoContent: {_0}")]
    NoContent(String),

    #[display(fmt = "ServiceUnavailable: {_0}")]
    ServiceUnavailable(String),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error. Please try later.")
            }
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::Conflict(ref message) => HttpResponse::Conflict().json(message),
            ServiceError::NoContent(ref message) => HttpResponse::NoContent().json(message),
            ServiceError::ServiceUnavailable(ref message) => {
                HttpResponse::ServiceUnavailable().json(message)
            }
        }
    }
}

impl From<ServiceError> for io::Error {
    fn from(err: ServiceError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{err:?}"))
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> ServiceError {
        ServiceError::NoContent(err.to_string())
    }
}

impl From<toml::de::Error> for ServiceError {
    fn from(err: toml::de::Error) -> ServiceError {
        ServiceError::NoContent(err.to_string())
    }
}

impl From<fast_log::error::LogError> for ServiceError {
    fn from(err: fast_log::error::LogError) -> ServiceError {
        ServiceError::Conflict(err.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for ServiceError {
    fn from(err: lettre::transport::smtp::Error) -> ServiceError {
        ServiceError::Conflict(err.to_string())
    }
}

impl From<lettre::address::AddressError> for ServiceError {
    fn from(err: lettre::address::AddressError) -> ServiceError {
        ServiceError::Conflict(err.to_string())
    }
}
