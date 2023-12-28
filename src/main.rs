use std::{
    fs,
    io::{self, BufRead},
    net::IpAddr,
    path::Path,
    str::FromStr,
};

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use lazy_static::lazy_static;
use log::{error, info};

pub mod api;
pub mod utils;

use std::process::exit;

use api::routes::post_mail;
use utils::{
    arg_parser::Args,
    config::{read_config, Config},
    ip_extrator::IpExtractor,
    logging::init_logger,
    mailer::{send, Msg},
};

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Config = read_config(&ARGS.config).expect("Missing Config");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger()?;

    if let Some(subject) = &ARGS.subject {
        match &ARGS.recipient {
            Some(recipient) => {
                let mut attachment = None;
                let mut attachment_name = None;

                if let Some(file) = &ARGS.attachment {
                    let size = fs::metadata(file)?.len();

                    if size > (CONFIG.max_attachment_size_mb * 1048576.0) as u64 {
                        eprintln!("Attachment to big!");
                        exit(1);
                    }
                    attachment = Some(fs::read(file)?);
                    attachment_name = Some(
                        Path::new(file)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    );
                }

                if let Some(text) = &ARGS.text {
                    let msg = Msg::new(
                        None,
                        recipient.clone(),
                        subject.clone(),
                        text.clone(),
                        attachment,
                        attachment_name,
                    );

                    send(msg).await?;
                } else {
                    let stdin = io::stdin();
                    let mut stdin_text = vec![];

                    for line in stdin.lock().lines() {
                        stdin_text.push(line?);
                    }

                    let msg = Msg::new(
                        None,
                        recipient.clone(),
                        subject.clone(),
                        stdin_text.join("\n"),
                        attachment,
                        attachment_name,
                    );

                    send(msg).await?;
                }
            }
            None => {
                eprintln!("No mail recipient available!");
                exit(1);
            }
        }

        return Ok(());
    }

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
