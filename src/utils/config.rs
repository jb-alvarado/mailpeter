use std::{fs, path::Path};

use log::LevelFilter;
use serde::{de, Deserialize, Deserializer};
use toml;

use crate::utils::errors::ServiceError;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub log_keep_count: i64,
    #[serde(deserialize_with = "string_to_log_level")]
    pub log_level: LevelFilter,
    pub log_size_mb: usize,
    pub log_to_file: bool,
    pub mail: Mail,
}

#[derive(Debug, Deserialize)]
pub struct Mail {
    pub smtp: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub starttls: bool,
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

pub fn read_config(path: &Option<String>) -> Result<Config, ServiceError> {
    let contents = fs::read_to_string(config_path(path))?;
    let data: Config = toml::from_str(&contents)?;

    Ok(data)
}
