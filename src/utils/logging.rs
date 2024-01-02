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
use crate::{ARGS, CONFIG};

/// The **LogFormat** struct is a custom log record formatter. It has two fields: **display_line_level**
/// and **time_type**. The **display_line_level** field is a **LevelFilter** that determines the minimum
/// level of log messages that should include the line number. The **time_type** field is a **TimeType**
///  that determines whether the timestamp in the log messages should be in local time or UTC.
pub struct LogFormat {
    pub display_line_level: LevelFilter,
    pub time_type: TimeType,
}

/// The **LogFormat** struct implements the RecordFormat trait from the **fast_log** crate. This trait
/// requires a **do_format** method that formats a **FastLogRecord** into a string.
///
/// The **do_format** method first checks if the **log_to_file** configuration option is enabled. If it
/// is, it formats the timestamp of the log record into a string. The format of the timestamp
/// depends on the **time_type** field. If **time_type** is **TimeType::Local**, the timestamp is converted
/// to local time. If **time_type** is **TimeType::Utc**, the timestamp is kept in UTC. The formatted
/// timestamp is then added to the beginning of the log message.
///
/// The method then checks the level of the log record. If the level is **LevelFilter::Trace**, it
/// includes the file name and line number in the log message. This is useful for debugging, as
/// it allows you to see exactly where the log message was generated. If the level is not
/// **LevelFilter::Trace**, it only includes the level and the message in the log message.
impl RecordFormat for LogFormat {
    fn do_format(&self, arg: &mut FastLogRecord) {
        match &arg.command {
            Command::CommandRecord => {
                let now = if CONFIG.log_to_file {
                    format!(
                        "[{: <29}] ",
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

/// The new method is a constructor for the **LogFormat** struct. It returns a new **LogFormat** with
/// **display_line_level** set to **LevelFilter::Debug** and **time_type** set to the default **TimeType**.
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

/// The **log_path** function returns the path to the log file. It first checks if the
/// **/var/log/mailpeter**, then fallbacks to **./logs** and finally to **./mailpeter.log**.
fn log_path() -> String {
    if Path::new("/var/log/mailpeter").is_dir() {
        return "/var/log/mailpeter/mailpeter.log".to_string();
    }

    if Path::new("./logs/mailpeter.log").is_dir() {
        return "./logs/mailpeter.log".to_string();
    }

    "./mailpeter.log".to_string()
}

/// The **init_logger** function initializes the logger. It first creates a **Config** struct with the log level set to the
/// **log_level** configuration option, the filter set to **ModuleFilter::new_exclude(vec!["rustls".to_string()])**,
/// and the formatter set to **LogFormat::new().set_display_line_level(CONFIG.log_level)**. It then checks if
/// **log_to_file** is enabled. If it is, it adds a file appender to the config. The file appender has the
/// path returned by **log_path**, the max log file size in megabytes set to the **log_size_mb** configuration
/// option, the max number of log files set to the **log_keep_count** configuration option, and the
/// **GZipPacker** plugin. If **log_to_file** is not enabled, it adds a console appender to the config.
/// Finally, it initializes the logger with the config.
pub fn init_logger() -> Result<(), ServiceError> {
    let level = if let Some(level) = &ARGS.level {
        match level.to_lowercase().as_str() {
            "debug" => LevelFilter::Debug,
            "error" => LevelFilter::Error,
            "info" => LevelFilter::Info,
            "trace" => LevelFilter::Trace,
            "warning" => LevelFilter::Warn,
            "off" => LevelFilter::Off,
            _ => {
                eprintln!("Log level not exists! Fallback to debug.");
                LevelFilter::Debug
            }
        }
    } else {
        CONFIG.log_level
    };

    let mut mail_config = Config::new()
        .chan_len(Some(100000))
        .level(level)
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
