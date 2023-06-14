// for CLI
use clap::Parser;

// logging
use colored::Colorize;
use log::debug;
use simple_logger::SimpleLogger;

// my mods
pub mod log_macros;

mod install;
pub mod ioc;
mod stage;

pub mod cli;
use cli::{Cli, Commands};

mod settings;
use settings::Settings;

mod file_system;
mod metadata;
mod shellbox;

use metadata::PackageData;

fn main() {
    let cli = Cli::parse();
    let config_file = cli.config_file.clone().unwrap_or("".to_string());
    let settings = Settings::build(&config_file).unwrap();

    // determine log level
    let dbg = settings.get_bool("debug").unwrap();
    let log_lvl = cli.get_level_filter(dbg);

    // initialize logging
    SimpleLogger::new()
        .with_level(log_lvl.to_owned())
        .init()
        .unwrap();

    let crate_info = PackageData::new();
    crate_info.report();

    // CLI commands
    match &cli.command {
        Some(Commands::Install {
            dryrun,
            retain,
            nodiff,
            all,
            force,
            iocs,
        }) => {
            debug!("command: <{}>", "install".yellow());
            debug!("dryrun: {}", dryrun);
            debug!("retain: {}", retain);
            debug!("no diff: {}", nodiff);
            debug!("all IOCs: {}", all);
            debug!("force:  {}", force);
            // worker
            install::ioc_install(iocs, &settings, dryrun, retain, nodiff, all, force);
        }
        Some(Commands::Stage { ioc }) => {
            debug!("stage: {:?}", ioc);
            stage::ioc_stage(ioc, None, &settings);
        }
        None => println!("no active command, check --help for more information."),
    }
}
