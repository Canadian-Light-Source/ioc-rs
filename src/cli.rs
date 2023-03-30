use clap::{Parser, Subcommand};

// CLI =================================================
#[derive(Parser)]
// #[command(name = "ioc")]
// #[command(author = "Niko Kivel <niko.kivel@lightsource.ca>")]
// #[command(version = "0.4.0")]
#[command(about = "Tool for the deployment of ioc definitions", long_about = None)]
pub struct Cli {
    /// Path to the tempalte directory
    #[arg(short, long)]
    pub template_dir: Option<String>,

    /// logger
    #[arg(short, long, default_value = "info")]
    pub log_level: Option<String>,

    /// config file
    #[arg(short, long)]
    pub config_file: Option<String>,

    /// The name of the command
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// delpoyment command
    Install {
        /// perform dryrun
        #[arg(short, long, action)]
        dryrun: bool,
        /// perform dryrun
        #[arg(long, action)]
        retain: bool,
        /// force install
        #[arg(short, long, action)]
        force: bool,
        /// list of IOCs to deploy, space separated
        #[clap(default_value = "", value_parser, num_args = 1.., value_delimiter = ' ')]
        iocs: Option<Vec<String>>,
    },
}
