use std::{net::IpAddr, str::FromStr};

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use lazy_static::lazy_static;
use log::{error, info};

pub mod api;
pub mod utils;

use api::routes::{post_mail, put_mail_attachment};
use utils::{
    arg_parser::Args,
    config::{read_config, Config},
    ip_extrator::IpExtractor,
    logging::init_logger,
    mailer::cli_sender,
};

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Config = read_config(&ARGS.config).expect("Missing Config");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger()?;

    if ARGS.subject.is_some() || ARGS.full_name.is_some() {
        // send mails from CLI
        if let Err(e) = cli_sender().await {
            eprintln!("{e}");
        }

        return Ok(());
    }

    let addr_port = match &ARGS.listen {
        // get listening IP first from arguments, then from config
        Some(ip) => ip,
        None => &CONFIG.listening_on,
    };

    if let Some((addr, port)) = addr_port.split_once(':') {
        info!("Running mailpeter, listen on http://{addr}:{port}");

        let trusted_proxy_ip = IpAddr::from_str(&CONFIG.reverse_proxy_ip).expect("Proxy IP");
        let mut enable_limit = false;
        let mut limit = 1;

        if CONFIG.limit_request_seconds > 0 {
            enable_limit = true;
            limit = CONFIG.limit_request_seconds;
        }

        // configure rate limit
        let governor_conf = GovernorConfigBuilder::default()
            .per_second(limit)
            .burst_size(1)
            .key_extractor(IpExtractor)
            .finish()
            .unwrap();

        HttpServer::new(move || {
            let mut app = App::new()
                .app_data(web::Data::new(trusted_proxy_ip))
                .wrap(middleware::Condition::new(
                    enable_limit,
                    Governor::new(&governor_conf),
                ))
                .wrap(middleware::Logger::new(
                    // custom logging format to get real IP behind proxy
                    "%{r}a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
                ));

            if CONFIG.routes.contains(&"text_only".to_string()) {
                // activate route for text and html messages, accept json format
                app = app.service(post_mail);
            }

            if CONFIG.routes.contains(&"with_attachments".to_string()) {
                // activate route with attachment support, accept multipart/form-data format
                app = app.service(put_mail_attachment);
            }

            app
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
