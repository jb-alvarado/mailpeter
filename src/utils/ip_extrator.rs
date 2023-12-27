use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use actix_governor::{
    governor::{clock::QuantaInstant, NotUntil},
    KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::{
    dev::ServiceRequest, http::header::ContentType, web, HttpResponse, HttpResponseBuilder,
};
use log::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpExtractor;

impl KeyExtractor for IpExtractor {
    type Key = IpAddr;

    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    #[cfg(feature = "log")]
    fn name(&self) -> &'static str {
        "real IP"
    }

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // Get the reverse proxy IP that we put in app data
        let reverse_proxy_ip = req
            .app_data::<web::Data<IpAddr>>()
            .map(|ip| ip.get_ref().to_owned())
            .unwrap_or_else(|| IpAddr::from_str("0.0.0.0").unwrap());

        let peer_ip = req.peer_addr().map(|socket| socket.ip());
        let connection_info = req.connection_info();

        match peer_ip {
            // The request is coming from the reverse proxy, we can trust the `Forwarded` or `X-Forwarded-For` headers
            Some(peer) if peer == reverse_proxy_ip => connection_info
                .realip_remote_addr()
                .ok_or_else(|| {
                    error!("Could not extract real IP address from request");
                    SimpleKeyExtractionError::new("Could not extract real IP address from request")
                })
                .and_then(|str| {
                    SocketAddr::from_str(str)
                        .map(|socket| socket.ip())
                        .or_else(|_| IpAddr::from_str(str))
                        .map_err(|_| {
                            SimpleKeyExtractionError::new(
                                "Could not extract real IP address from request",
                            )
                        })
                }),
            // The request is not coming from the reverse proxy, we use peer IP
            _ => connection_info
                .peer_addr()
                .ok_or_else(|| {
                    error!("Could not extract peer IP address from request");
                    SimpleKeyExtractionError::new("Could not extract peer IP address from request")
                })
                .and_then(|str| {
                    SocketAddr::from_str(str)
                        .map(|socket| socket.ip())
                        .or_else(|_| IpAddr::from_str(str))
                        .map_err(|e| {
                            error!("{e}: {str}");
                            SimpleKeyExtractionError::new(
                                "Could not extract peer IP address from request",
                            )
                        })
                }),
        }
    }

    // This function is only needed, because we removing the seconds to wait.
    // If the original message is needed, remove the hole function.
    fn exceed_rate_limit_response(
        &self,
        _negative: &NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        // let wait_time = negative
        //     .wait_time_from(DefaultClock::default().now())
        //     .as_secs();
        response
            .content_type(ContentType::plaintext())
            // Original response string is:
            //     `format!("Too many requests, retry in {}s", wait_time)`
            // but better we hide the seconds from user.
            .body("Too many requests")
    }

    #[cfg(feature = "log")]
    fn key_name(&self, key: &Self::Key) -> Option<String> {
        Some(key.to_string())
    }
}
