use std::{env, path::Path};

use colored::Colorize;
use config::Config;
use log::{debug, error, info, trace};
use std::fs;

use crate::{
    hash_ioc,
    ioc::IOC,
    log_macros::{cross, exclaim, tick},
    stage,
};

// TODO: move to function
pub fn ioc_install(
    iocs: &Option<Vec<String>>,
    settings: &Config,
    dryrun: &bool,
    retain: &bool,
    nodiff: &bool,
    force: &bool,
) {
    let stage_root = settings.get::<String>("filesystem.stage").unwrap();
    let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
    let template_dir = settings.get::<String>("app.template_directory").unwrap();

    trace!("configuration ---------------------------");
    trace!("  stage:    {:?}", stage_root);
    trace!("  deploy:   {:?}", deploy_root);
    trace!("  templates:{:?}", template_dir);
    trace!("-----------------------------------------");

    let ioc_list = match iocs {
        Some(i) => collect_iocs(i, &stage_root, &deploy_root),
        None => panic!(),
    };
    trace!("{} ioc list created", tick!());

    for ioc in &ioc_list {
        info!("----- {} -----", ioc.name.blue().bold());
        trace!("{:?}", ioc);
        // temper check
        match hash_ioc::check_hash(ioc, force) {
            Ok(_hash) => {}
            Err(e) => {
                error!(
                    "{} {}: aborting deployment of {}",
                    cross!(),
                    e,
                    ioc.name.red().bold()
                );
                continue;
            }
        }
        // staging
        trace!("staging {}", ioc.name.blue().bold());
        stage::ioc_stage(&None, Some(ioc.clone()), settings);
        if ioc.destination.exists() && !nodiff {
            match ioc.diff_ioc() {
                Ok(_) => info!("{} diffed {} see output above", tick!(), ioc.name.blue()),
                Err(e) => error!(
                    "{} diff of {} failed with: {}",
                    cross!(),
                    ioc.name.red().bold(),
                    e
                ),
            }
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
            };
            match ioc_cleanup(ioc) {
                Ok(_) => {}
                Err(e) => error!(
                    "{} clean up failed for {} with error: {}",
                    cross!(),
                    &ioc.name,
                    e
                ),
            };
            match remove_dir(Path::new(&stage_root)) {
                Ok(_) => info!("{} stage root removed", tick!()),
                Err(e) => {
                    error!("{} failed to remove stage root with error: {}", cross!(), e)
                }
            };
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

fn remove_dir(dir: impl AsRef<Path>) -> std::io::Result<()> {
    trace!("removing directory {}", dir.as_ref().to_str().unwrap());
    fs::remove_dir_all(dir)?;
    Ok(())
}
