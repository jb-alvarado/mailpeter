use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(version,
    about = "Rust Contact API",
    long_about = None)]
pub struct Args {
    #[clap(short = 'A', long, help = "Path to attachment file")]
    pub attachment: Option<String>,

    #[clap(short, long, help = "Path to config")]
    pub config: Option<String>,

    #[clap(short, long, help = "Listen on IP:PORT, like: 127.0.0.1:8989")]
    pub listen: Option<String>,

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
}
