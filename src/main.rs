use std::env;
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

#[cfg(test)]
mod test_utils;

fn main() {
    let cli = Cli::parse();
    let config_file = cli.config_file.clone().unwrap_or("".to_string());
    let settings = Settings::build(&config_file).unwrap();

    // determine log level
    let log_lvl = cli.get_level_filter();

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
            let work_dir = env::current_dir().unwrap().join(ioc.as_ref().unwrap());
            // let stage_root = settings.get::<String>("filesystem.stage").unwrap();
            // let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
            // let template_dir = settings.get::<String>("app.template_directory").unwrap();
            // let ioc_struct = ioc::IOC::new(&work_dir, &stage_root, &deploy_root, &template_dir)
            //     .expect("from_list failed");
            // stage::ioc_stage(&None, Some(ioc_struct), &settings);
            let ioc_struct = ioc::IOC::new_with_settings(work_dir, &settings);
            let _ = stage::stage(&ioc_struct);
        }
        None => println!("no active command, check --help for more information."),
    }
}
