use std::io;

use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use log::error;
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
        error!("{err:?}");

        io::Error::new(io::ErrorKind::Other, format!("{err:?}"))
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> ServiceError {
        error!("{err:?}");

        ServiceError::NoContent(err.to_string())
    }
}

impl From<toml::de::Error> for ServiceError {
    fn from(err: toml::de::Error) -> ServiceError {
        error!("{err:?}");

        ServiceError::InternalServerError
    }
}

impl From<fast_log::error::LogError> for ServiceError {
    fn from(err: fast_log::error::LogError) -> ServiceError {
        error!("{err:?}");

        ServiceError::InternalServerError
    }
}

impl From<lettre::transport::smtp::Error> for ServiceError {
    fn from(err: lettre::transport::smtp::Error) -> ServiceError {
        error!("{err:?}");

        ServiceError::Conflict(err.to_string())
    }
}

impl From<lettre::transport::file::Error> for ServiceError {
    fn from(err: lettre::transport::file::Error) -> ServiceError {
        error!("{err:?}");

        ServiceError::InternalServerError
    }
}

impl From<lettre::address::AddressError> for ServiceError {
    fn from(err: lettre::address::AddressError) -> ServiceError {
        error!("{err:?}");

        ServiceError::Conflict(err.to_string())
    }
}

impl From<lettre::error::Error> for ServiceError {
    fn from(err: lettre::error::Error) -> ServiceError {
        error!("{err:?}");

        ServiceError::InternalServerError
    }
}

impl From<std::net::AddrParseError> for ServiceError {
    fn from(err: std::net::AddrParseError) -> ServiceError {
        error!("{err:?}");

        ServiceError::InternalServerError
    }
}

impl From<actix_multipart::MultipartError> for ServiceError {
    fn from(err: actix_multipart::MultipartError) -> ServiceError {
        error!("{err:?}");

        ServiceError::BadRequest(err.to_string())
    }
}
