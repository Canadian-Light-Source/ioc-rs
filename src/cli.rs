use clap::{Parser, Subcommand};
use log::LevelFilter;

// CLI =================================================
#[derive(Parser, Clone)]
#[command(about = "Tool for the deployment of ioc definitions", long_about = None)]
pub struct Cli {
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

impl Cli {
    /// returns log level filter based on the command line arguments, if debug is enabled in the configuration, the CLI is overridden
    pub fn get_level_filter(&self, debug: bool) -> LevelFilter {
        let mut cli_level = self.log_level.as_ref().unwrap().to_lowercase();

        if debug {
            cli_level = "debug".to_string()
        };
        if cli_level == "trace" {
            LevelFilter::Trace
        } else if cli_level == "debug" {
            LevelFilter::Debug
        } else if cli_level == "warn" {
            LevelFilter::Warn
        } else if cli_level == "info" {
            LevelFilter::Info
        } else {
            LevelFilter::Error
        } // always report errors
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// delpoyment command
    Install {
        /// perform dryrun
        #[arg(short, long, action)]
        dryrun: bool,
        /// retain staging data
        #[arg(long, action)]
        retain: bool,
        /// do not show the diff
        #[arg(long, action)]
        nodiff: bool,
        /// force install
        #[arg(short, long, action)]
        force: bool,
        /// list of IOCs to deploy, space separated
        #[clap(default_value = "", value_parser, num_args = 1.., value_delimiter = ' ')]
        iocs: Option<Vec<String>>,
    },
}
