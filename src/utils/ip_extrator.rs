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

/// This struct doesn't have any fields, it's just a marker that implements the **KeyExtractor** trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpExtractor;

/// The **KeyExtractor** trait has two associated types: **Key** and **KeyExtractionError**. For **IpExtractor**,
/// **Key** is **IpAddr**, which means the key is an IP address. **KeyExtractionError** is
/// **SimpleKeyExtractionError<&'static str>**, which means the error type is a simple error with a
/// static string message.
///
/// The **name** method returns a static string that is the name of the key extractor. This is only
/// compiled when the "log" feature is enabled.
///
/// The **extract** method is where the IP address is extracted from the request. It first gets the reverse
/// proxy IP from the app data. If the app data doesn't contain an IP address, it defaults to "0.0.0.0".
/// It then gets the peer IP from the request, which is the IP address of the client that made the request.
///
/// The method then checks if the peer IP is the same as the reverse proxy IP. If it is, it means the
/// request is coming from the reverse proxy, so it tries to get the real IP from the **Forwarded** or
/// **X-Forwarded-For** headers. If it can't get the real IP, it logs an error and returns a
/// **SimpleKeyExtractionError**.
///
/// If the peer IP is not the same as the reverse proxy IP, it means the request is not coming from the reverse
/// proxy, so it uses the peer IP as the key. If it can't get the peer IP, it logs an error and returns a
/// **SimpleKeyExtractionError**.

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

    // This function is only needed because we are removing the seconds to wait.
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
