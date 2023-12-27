use std::{net::IpAddr, str::FromStr};

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use lazy_static::lazy_static;
use log::{error, info};

pub mod api;
pub mod utils;

use api::routes::post_mail;
use utils::{
    arg_parser::Args,
    config::{read_config, Config},
    ip_extrator::IpExtractor,
    logging::init_logger,
};

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Config = read_config(&ARGS.config).expect("Missing Config");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger()?;

    if let Some((addr, port)) = ARGS.listen.clone().unwrap_or_default().split_once(':') {
        info!("Running mailpeter, listen on http://{addr}:{port}");

        let trusted_proxy_ip = IpAddr::from_str(&CONFIG.reverse_proxy_ip).expect("Proxy IP");
        let governor_conf = GovernorConfigBuilder::default()
            .per_second(CONFIG.limit_request_seconds)
            .burst_size(1)
            .key_extractor(IpExtractor)
            .finish()
            .unwrap();

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(trusted_proxy_ip))
                .wrap(Governor::new(&governor_conf))
                // custom logger to get client IPs behind Proxy
                .wrap(middleware::Logger::new(
                    "%{r}a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
                ))
                .service(post_mail)
        })
        .bind((addr.to_string(), port.parse().unwrap_or_default()))?
        .run()
        .await
    } else {
        error!("Run mailpeter with listen parameter!");
        log::logger().flush();
        Ok(())
    }
}
