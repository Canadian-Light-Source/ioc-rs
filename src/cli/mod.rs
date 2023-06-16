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
    /// returns log level filter based on the command line arguments
    pub fn get_level_filter(&self) -> LevelFilter {
        let cli_level = self.log_level.as_ref().unwrap().to_lowercase();

        if cli_level == "trace" {
            LevelFilter::Trace
        } else if cli_level == "debug" {
            LevelFilter::Debug
        } else if cli_level == "info" {
            LevelFilter::Info
        } else if cli_level == "warn" {
            LevelFilter::Warn
        } else {
            LevelFilter::Error
        } // always report errors
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// deployment command
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

        /// install _all_ IOCs in PWD
        #[arg(short, long, action)]
        all: bool,

        /// force install
        #[arg(short, long, action)]
        force: bool,

        /// list of IOCs to deploy, space separated. Excludes `--all`!
        // #[clap(default_value = "", value_parser, num_args = 1.., value_delimiter = ' ')]
        #[clap(value_parser, num_args = 1.., value_delimiter = ' ')]
        iocs: Option<Vec<String>>,
    },
    Stage {
        /// single IOCs to stage
        #[clap()]
        ioc: String,
        /// optional staging directory
        // #[clap(default_value = "")]
        #[arg(short, long)]
        path: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    // test proper retrieval of log level
    #[test]
    fn log_level() {
        let mut test_cli = Cli {
            log_level: None,
            config_file: None,
            command: None,
        };
        // fallback to "Error"
        test_cli.log_level = Some("foobar".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Error);
        // case insensitive
        test_cli.log_level = Some("eRrOr".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Error);

        test_cli.log_level = Some("error".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Error);

        test_cli.log_level = Some("warn".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Warn);

        test_cli.log_level = Some("info".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Info);

        test_cli.log_level = Some("debug".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Debug);

        test_cli.log_level = Some("trace".to_string());
        assert_eq!(test_cli.get_level_filter(), LevelFilter::Trace);
    }
}
