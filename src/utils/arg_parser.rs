use clap::Parser;

/// Define the command line arguments
#[derive(Parser, Debug, Clone)]
#[clap(version,
    about = "Rust Contact API",
    long_about = None)]
pub struct Args {
    #[clap(short = 'A', long, num_args = 0.., help = "Path to attachment file")]
    pub attachment: Option<Vec<String>>,

    #[clap(short, long, help = "Path to config")]
    pub config: Option<String>,

    // Hidden unused parameter for sendmail compatibility
    #[clap(short = 'B', long, hide = true)]
    pub body_type: Option<String>,

    #[clap(
        short = 'F',
        long,
        help = "Set the sender full name, this override From header"
    )]
    pub full_name: Option<String>,

    // Hidden unused parameter for sendmail compatibility
    #[clap(short, long, hide = true)]
    pub ignore: bool,

    #[clap(short, long, help = "Listen on IP:PORT, like: 127.0.0.1:8989")]
    pub listen: Option<String>,

    #[clap(
        short = 'L',
        long,
        help = "Log level, like: debug, info, warn, error, off"
    )]
    pub level: Option<String>,

    #[clap(help = "Mail recipient for command line usage")]
    pub recipient: Option<String>,

    #[clap(short, long, help = "Mail subject for command line usage")]
    pub subject: Option<String>,

    #[clap(
        short,
        long,
        help = "Mail text for command line usage, stdin without -t work too"
    )]
    pub text: Option<String>,

    // Hidden unused parameter for sendmail compatibility
    #[clap(short, long, hide = true)]
    pub ox: Option<String>,
}
