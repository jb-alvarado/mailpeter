use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(version,
    about = "Rust Contact API",
    long_about = None)]
pub struct Args {
    #[clap(long, help = "Path to config")]
    pub config: Option<String>,

    #[clap(short, long, help = "Listen on IP:PORT, like: 127.0.0.1:8989")]
    pub listen: Option<String>,
}
