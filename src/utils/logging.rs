use std::path::Path;

use fast_log::{
    appender::{Command, FastLogRecord, RecordFormat},
    consts::LogSize,
    filter::ModuleFilter,
    plugin::{file_split::KeepType, packer::GZipPacker},
    Config, TimeType,
};
use log::LevelFilter;

use crate::utils::errors::ServiceError;
use crate::CONFIG;

pub struct LogFormat {
    pub display_line_level: LevelFilter,
    pub time_type: TimeType,
}

impl RecordFormat for LogFormat {
    fn do_format(&self, arg: &mut FastLogRecord) {
        match &arg.command {
            Command::CommandRecord => {
                let now = if CONFIG.log_to_file {
                    format!(
                        "[{:27}] ",
                        match self.time_type {
                            TimeType::Local => fastdate::DateTime::from(arg.now)
                                .set_offset(fastdate::offset_sec())
                                .display_stand(),
                            TimeType::Utc => fastdate::DateTime::from(arg.now).display_stand(),
                        }
                    )
                } else {
                    String::new()
                };

                if arg.level.to_level_filter() == LevelFilter::Trace {
                    arg.formated = format!(
                        "{}[{: >5}] {}:{} {}\n",
                        &now,
                        arg.level,
                        arg.file,
                        arg.line.unwrap_or_default(),
                        arg.args,
                    );
                } else {
                    arg.formated = format!("{}[{: >5}] {}\n", &now, arg.level, arg.args);
                }
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
    }
}

impl LogFormat {
    pub fn new() -> Self {
        Self {
            display_line_level: LevelFilter::Debug,
            time_type: TimeType::default(),
        }
    }

    pub fn set_display_line_level(mut self, level: LevelFilter) -> Self {
        self.display_line_level = level;
        self
    }
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::new()
    }
}

fn log_path() -> String {
    if Path::new("/var/log/mailpeter").is_dir() {
        return "/var/log/mailpeter/mailpeter.log".to_string();
    }

    if Path::new("./logs/mailpeter.log").is_dir() {
        return "./logs/mailpeter.log".to_string();
    }

    "./mailpeter.log".to_string()
}

pub fn init_logger() -> Result<(), ServiceError> {
    let mut mail_config = Config::new()
        .chan_len(Some(100000))
        .level(CONFIG.log_level)
        .filter(ModuleFilter::new_exclude(vec!["rustls".to_string()]))
        .format(LogFormat::new().set_display_line_level(CONFIG.log_level));

    if CONFIG.log_to_file {
        mail_config = mail_config.file_split(
            &log_path(),
            LogSize::MB(CONFIG.log_size_mb),
            KeepType::KeepNum(CONFIG.log_keep_count),
            GZipPacker {},
        );
    } else {
        mail_config = mail_config.console();
    }

    fast_log::init(mail_config)?;

    Ok(())
}
