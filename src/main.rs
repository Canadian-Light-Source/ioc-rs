use std::io::{self, Error};

use std::env;
use std::path::Path;
use std::process::exit;
// for CLI
use clap::{CommandFactory, Parser};

// logging
use colored::Colorize;
use log::{debug, error, info, trace, warn};
use simple_logger::SimpleLogger;

// my mods
pub mod cli;
use cli::{Cli, Commands};
mod install;
pub mod ioc;
pub mod log_macros;
mod stage;

mod settings;
use settings::Settings;
mod file_system;
mod metadata;
mod origin;
pub mod shellbox;

use crate::log_macros::{cross, tick};
use metadata::PackageData;

#[cfg(test)]
mod test_utils;
fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {generator:?}...");
        cli::print_completions(generator, &mut cmd);
        exit(0);
    }

    if cli.ver {
        let m = metadata::PackageData::new();
        println!("{} {}", m.get_name(), m.get_version());
        exit(0);
    }

    let config_file = cli.config_file.clone().unwrap_or("".to_string());

    // determine log level
    let log_lvl = cli.get_level_filter();

    // initialize logging
    SimpleLogger::new()
        .with_level(log_lvl.to_owned())
        .init()
        .unwrap();

    let settings = Settings::build(&config_file).unwrap();
    /* TODO: This still very sub-optimal. In this early stage, a std::process::exit would do.
        Maybe the tera instance could be created here already?
    */
    match Settings::verify(&settings) {
        Ok(_) => trace!("{} verified {}", tick!(), config_file),
        Err(e) => {
            error!("{} config file verification failed with\n {e}!", cross!());
            exit(1)
        }
    };

    let crate_info = PackageData::new();
    crate_info.report();

    // CLI commands
    match &cli.command {
        Some(Commands::Install(args)) => {
            debug!("command: <{}>", "install".yellow());
            debug!("dryrun: {}", args.dryrun);
            debug!("no diff: {}", args.nodiff);
            debug!("force:  {}", args.force);
            // worker
            install::ioc_install(
                &args.iocs,
                &settings,
                &args.dryrun,
                &args.nodiff,
                &args.force,
            )?;
            Ok(())
        }
        Some(Commands::Uninstall(args)) => {
            let source = Path::new(&args.ioc);
            let stage_root = match settings.get::<String>("filesystem.stage") {
                Ok(env_key) => match env::var(&env_key) {
                    Ok(val) => val + "/ioc/stage",
                    Err(_) => "/tmp/ioc/stage".to_string(), // Env var not set
                },
                Err(_) => "/tmp/ioc/stage".to_string(), // YAML path not found or wrong type
            };

            let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
            let shellbox_root = settings.get::<String>("filesystem.shellbox").unwrap();
            let template_dir = settings.get::<String>("app.template_directory").unwrap();
            let ioc_struct = match ioc::IOC::new(
                source,
                &stage_root,
                &deploy_root,
                &shellbox_root,
                &template_dir,
            ) {
                Ok(ioc) => ioc,
                Err(e) => {
                    error!("{} failed to build IOC with: {}", cross!(), e.red());
                    return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid list"));
                }
            };
            let message = format!("\
            Uninstalling IOCs is not yet supported. Follow the steps below for manual uninstallation\n\
            To uninstall >>{}<< you need to:\n\
              - remove the line for port {} in {}{}/shellbox.conf\n\
              - run the command \'shellbox reload\' on {}\n\
              - delete {}{}",
                &args.ioc.red().bold(),
                &ioc_struct.config.ioc.port,
                &shellbox_root,
                &ioc_struct.config.ioc.host,
                &ioc_struct.config.ioc.host,
                &deploy_root,
                &ioc_struct.name,
            );
            warn!("command: <{}>", "uninstall".yellow());
            warn!("{message}");
            Ok(())
        }
        Some(Commands::Stage(args)) => {
            info!("----- {} -----", args.ioc.blue().bold());
            let source = Path::new(&args.ioc);
            let stage_root = match settings.get::<String>("filesystem.stage") {
                Ok(env_key) => match env::var(&env_key) {
                    Ok(val) => val + "/ioc/stage",
                    Err(_) => "/tmp/ioc/stage".to_string(), // Env var not set
                },
                Err(_) => "/tmp/ioc/stage".to_string(), // YAML path not found or wrong type
            };

            let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
            let shellbox_root = settings.get::<String>("filesystem.shellbox").unwrap();
            let template_dir = settings.get::<String>("app.template_directory").unwrap();
            match ioc::IOC::new(
                source,
                &stage_root,
                &deploy_root,
                &shellbox_root,
                &template_dir,
            ) {
                Ok(ioc) => stage::stage(&ioc)?,
                Err(e) => {
                    error!("{} failed to build IOC with: {}", cross!(), e.red());
                    // return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid list"));
                }
            };
            Ok(())
        }
        None => {
            let error_msg = "no active command, check --help for more information.";
            error!("{} {}", cross!(), error_msg);
            exit(1);
        }
    }
}
