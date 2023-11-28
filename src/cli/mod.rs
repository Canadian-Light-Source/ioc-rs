use std::io;
// use clap::{Parser, Subcommand};
use clap::{Args, Command, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Generator, Shell};
use log::LevelFilter;

// CLI =================================================
// #[derive(Parser, Clone)]
#[derive(Parser, Debug, PartialEq)]
// #[command(name = "ioc")]
#[command(
    name = "ioc",
    about = "Tool for the deployment of ioc definitions",
    long_about = "Tool for the deployment of ioc definitions\nhttps://github.lightsource.ca/epics-iocs/ioc-rs"
)]
pub struct Cli {
    /// shell tab complete generator
    #[arg(long = "generate", value_enum)]
    pub generator: Option<Shell>,

    /// display version
    #[arg(short, long, action)]
    pub ver: bool,

    /// log level: error, warn, info, debug, trace
    #[arg(short, long, default_value = "info")]
    pub log_level: Option<String>,

    /// config file
    #[arg(short, long)]
    pub config_file: Option<String>,

    /// The name of the command
    #[command(subcommand)]
    pub command: Option<Commands>,
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
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

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    // #[command(visible_alias = "hint")]
    // ValueHint(StageCommand),
    #[clap(name = "install")]
    Install(InstallCommand),
    #[clap(name = "stage")]
    Stage(StageCommand),
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct InstallCommand {
    /// perform dryrun
    #[arg(short, long, action)]
    pub dryrun: bool,

    /// do not show the diff
    #[arg(long, action)]
    pub nodiff: bool,

    /// force install
    #[arg(short, long, action)]
    pub force: bool,

    /// list of IOCs to deploy, space separated. Excludes `--all`!
    // #[clap(default_value = "", value_parser, num_args = 1.., value_delimiter = ' ')]
    #[clap(value_parser, num_args = 1.., value_delimiter = ' ')]
    pub iocs: Option<Vec<String>>,
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct StageCommand {
    /// single IOCs to stage
    #[clap(value_hint = ValueHint::DirPath)]
    pub ioc: String,
    /// optional staging directory
    // #[clap(default_value = "")]
    #[arg(short, long, value_hint = ValueHint::DirPath)]
    pub path: Option<String>,
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    // test proper retrieval of log level
    #[test]
    fn log_level() {
        let mut test_cli = Cli {
            generator: None,
            ver: false,
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
