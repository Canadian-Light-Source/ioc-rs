use std::io::{self, Error, ErrorKind};

use std::path::Path;
// for CLI
use clap::Parser;

// logging
use colored::Colorize;
use log::{debug, error, info};
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

use crate::log_macros::cross;
use metadata::PackageData;

#[cfg(test)]
mod test_utils;

fn main() -> io::Result<()> {
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
            nodiff,
            force,
            iocs,
        }) => {
            debug!("command: <{}>", "install".yellow());
            debug!("dryrun: {}", dryrun);
            debug!("no diff: {}", nodiff);
            debug!("force:  {}", force);
            // worker
            install::ioc_install(iocs, &settings, dryrun, nodiff, force);
            Ok(())
        }
        Some(Commands::Stage { ioc, path }) => {
            debug!("stage: {:?}", ioc);
            let source = Path::new(ioc);
            let stage_root = match path {
                Some(p) => {
                    info!("staging in {}", p);
                    p.clone()
                }
                None => settings.get::<String>("filesystem.stage").unwrap(),
            };
            let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
            let template_dir = settings.get::<String>("app.template_directory").unwrap();
            let ioc_struct = ioc::IOC::new(source, &stage_root, &deploy_root, &template_dir)
                .expect("from_list failed");
            stage::stage(&ioc_struct)?;
            Ok(())
        }
        None => {
            let error_msg = "no active command, check --help for more information.";
            error!("{} {}", cross!(), error_msg);
            let e = Error::new(ErrorKind::Other, error_msg);
            Err(e)
        }
    }
}
