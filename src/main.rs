use std::env;
use std::fs;
use std::path::Path;

// for CLI
use clap::Parser;

// logging
use colored::Colorize;
use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use simple_logger::SimpleLogger;

// my mods
pub mod ioc;
use ioc::IOC;

pub mod cli;
use cli::{Cli, Commands};

mod settings;
use settings::Settings;

pub mod log_macros;
use crate::log_macros::{cross, exclaim, tick};

fn collect_iocs(
    ioc_names: &[String],
    stage_root: impl AsRef<Path>,
    destination_root: impl AsRef<Path>,
) -> Vec<IOC> {
    let mut iocs: Vec<IOC> = Vec::new();
    debug!("collecting iocs ...");
    ioc_names.iter().for_each(|name| {
        let work_dir = env::current_dir().unwrap().join(name);
        trace!("working dir: {:?}", work_dir);
        match IOC::new(&work_dir, &stage_root, &destination_root) {
            Ok(new_ioc) => iocs.push(new_ioc),
            Err(e) => error!(
                "{} IOC::new failed for <{}> with: {}",
                cross!(),
                name.red().bold(),
                e
            ),
        };
    });
    iocs
}

fn ioc_cleanup(ioc: &IOC) -> std::io::Result<()> {
    trace!("cleaning up staging directory for {}", &ioc.name);
    fs::remove_dir_all(&ioc.stage)?;
    info!("{} cleaning up: removed {:?}", tick!(), &ioc.stage);
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let settings = Settings::build(&cli.config_file.unwrap_or("".to_string())).unwrap();

    // determine log level
    let mut l = cli.log_level.unwrap().to_lowercase();
    let dbg = settings.get_bool("debug").unwrap_or(false);
    // orverride log level with configuration file
    if dbg {
        l = "trace".to_string()
    };
    let log_lvl = if l == "trace" {
        LevelFilter::Trace
    } else if l == "debug" {
        LevelFilter::Debug
    } else if l == "warn" {
        LevelFilter::Warn
    } else if l == "info" {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    }; // always report errors

    // initialize logging
    SimpleLogger::new().with_level(log_lvl).init().unwrap();

    let stage_root = settings
        .get::<String>("filesystem.stage")
        .unwrap_or("stage/".to_string());
    let deploy_root = settings
        .get::<String>("filesystem.deploy")
        .unwrap_or("deploy/ioc/".to_string());
    let template_dir = settings
        .get::<String>("app.template_directory")
        .unwrap_or("templates/*.tera".to_string());

    trace!("configuration ---------------------------");
    trace!("  stage:    {:?}", stage_root);
    trace!("  deploy:   {:?}", deploy_root);
    trace!("  templates:{:?}", template_dir);
    trace!("-----------------------------------------");

    // template directory from cli, defaults to configuration files
    let template_dir = &cli.template_dir.unwrap_or(template_dir);
    trace!("template_dir: {:?}", template_dir);

    // CLI commands
    // TODO: terribly nested, check for way to flatten
    match &cli.command {
        Some(Commands::Install {
            dryrun,
            retain,
            force,
            iocs,
        }) => {
            debug!("command: <{}>", "install".yellow());
            debug!("dryrun: {}", dryrun);
            debug!("retain: {}", retain);
            debug!("force:  {}", force);
            let ioc_list = match iocs {
                Some(i) => collect_iocs(i, &stage_root, &deploy_root),
                None => panic!(),
            };
            trace!("{} ioc list created", tick!());
            // worker
            // TODO: move to function
            for ioc in &ioc_list {
                info!("----- {} -----", ioc.name.blue().bold());
                trace!("{:?}", ioc);
                // temper check
                match ioc.check_hash() {
                    Ok(hash) => {
                        info!("{} valid hash for {} |{}|", tick!(), &ioc.name.blue(), hash);
                    }
                    Err(e) => {
                        if !force {
                            error!("{} {} --> check destination <{:?}> and use `{} {}` to deploy regardless", cross!(), e, &ioc.destination.as_path(), "ioc install --force".yellow(), &ioc.name.yellow());
                            continue;
                        } else {
                            warn!(
                                "{} failed hash check overwritten by {}",
                                exclaim!(),
                                "--force".yellow()
                            );
                        }
                    }
                }
                // staging
                trace!("staging {}", ioc.name.blue().bold());
                match ioc.stage(template_dir) {
                    Ok(_) => info!("{} staged {}", tick!(), ioc.name.blue()),
                    Err(e) => error!(
                        "{} staging of {} failed with: {}",
                        cross!(),
                        ioc.name.red().bold(),
                        e
                    ),
                }
                // deployment
                if !dryrun {
                    trace!("deploying {}", ioc.name.blue().bold());
                    match ioc.deploy() {
                        Ok(_) => info!("{} deployed {}", tick!(), ioc.name.blue()),
                        Err(e) => error!(
                            "{} deployment of {} failed with: {}",
                            cross!(),
                            ioc.name.red().bold(),
                            e
                        ),
                    }
                } else {
                    info!("{} was chosen, no deployment", "--dryrun".yellow());
                    if !retain {
                        match ioc_cleanup(ioc) {
                            Ok(_) => {}
                            Err(e) => error!(
                                "{} clean up failed for {} with error: {}",
                                cross!(),
                                &ioc.name,
                                e
                            ),
                        };
                    } else {
                        info!(
                            "{} stage directory retained. Make sure to clean up after yourself!",
                            exclaim!()
                        );
                    }
                }
                trace!("------------");
            }
        }
        None => println!("NO ACTION --> BYE"),
    }
}
