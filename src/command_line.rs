use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[arg(short = 'p', long, default_value = "9104")]
    pub port: u16,

    #[arg(short = 'H', long, help = "specify the hostname", default_value = "0.0.0.0")]
    pub host: String,

    #[arg(short = 't', long, help = "list of target files to monitor, semicolon separated")]
    pub targets: String,

    #[command(flatten)]
    pub verbosity: Verbosity,
}
