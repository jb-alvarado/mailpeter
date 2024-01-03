use std::{fs, path::Path};

use log::{debug, LevelFilter};
use serde::{de, Deserialize, Deserializer};
use toml;

use crate::utils::errors::ServiceError;

/// Config structs

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listening_on: String,
    pub log_keep_count: i64,
    #[serde(deserialize_with = "string_to_log_level")]
    pub log_level: LevelFilter,
    pub log_size_mb: usize,
    pub log_to_file: bool,
    pub reverse_proxy_ip: String,
    pub limit_request_seconds: u64,
    pub max_attachment_size_mb: f64,
    pub routes: Vec<String>,
    pub mail_archive: String,
    pub mail: Mail,
}

#[derive(Debug, Deserialize)]
pub struct Mail {
    pub smtp: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub starttls: bool,
    pub alias: String,
    pub block_words: Vec<String>,
    pub recipients: Vec<Recipients>,
}

#[derive(Debug, Deserialize)]
pub struct Recipients {
    pub direction: String,
    pub mails: Vec<String>,
    #[serde(skip_deserializing)]
    pub subject: String,
    #[serde(skip_deserializing)]
    pub message: String,
}

/// Deserialize log level from string
pub fn string_to_log_level<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    match s.to_lowercase().as_str() {
        "debug" => Ok(LevelFilter::Debug),
        "error" => Ok(LevelFilter::Error),
        "info" => Ok(LevelFilter::Info),
        "trace" => Ok(LevelFilter::Trace),
        "warning" => Ok(LevelFilter::Warn),
        "off" => Ok(LevelFilter::Off),
        _ => Err(de::Error::custom("Log level not exists!")),
    }
}

/// Get config file path, fallback to different locations
pub fn config_path(path: &Option<String>) -> String {
    if let Some(p) = path {
        return p.clone();
    }

    if Path::new("/etc/mailpeter/mailpeter.toml").is_file() {
        return "/etc/mailpeter/mailpeter.toml".to_string();
    }

    if Path::new("mailpeter.toml").is_file() {
        return "mailpeter.toml".to_string();
    }

    "./assets/mailpeter.toml".to_string()
}

/// read config from file
pub fn read_config(path: &Option<String>) -> Result<Config, ServiceError> {
    let config_file = config_path(path);
    debug!("Read config from: {}", config_file);

    let contents = fs::read_to_string(config_file)?;
    let data: Config = toml::from_str(&contents)?;

    Ok(data)
}
