use std::collections::HashSet;
use std::path::{Path, PathBuf};

use colored::Colorize;
use config::Config;
use glob::glob;
use log::{debug, error, info, trace};
use std::{fs, io};

use crate::shellbox::ioc_shellbox;
use crate::{
    ioc::hash_ioc,
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
    all: &bool,
    force: &bool,
) {
    let ioc_list = check_ioc_list(iocs, *all);
    let stage_root = settings.get::<String>("filesystem.stage").unwrap();
    let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
    let template_dir = settings.get::<String>("app.template_directory").unwrap();

    trace!("configuration ---------------------------");
    trace!("  stage:    {:?}", stage_root);
    trace!("  deploy:   {:?}", deploy_root);
    trace!("  templates:{:?}", template_dir);
    trace!("-----------------------------------------");

    let ioc_list = IOC::from_list(&ioc_list, &stage_root, &deploy_root);
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
        if ioc.destination.exists() && !*nodiff {
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

        // TODO: error handler
        let _ = ioc_shellbox(ioc, settings);

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

fn check_ioc_list(list: &Option<Vec<String>>, all: bool) -> Vec<String> {
    match list {
        Some(_l) if all => {
            error!(
                "{} {} is exclusive to empty list of IOCs.",
                cross!(),
                "--all".bold().yellow()
            );
            panic!("--all is exclusive to empty list of IOCs")
        }
        Some(l) if !l.is_empty() => filter_duplicates(l.clone()).unwrap(),
        None => {
            if !all {
                error!("{} empty list iof IOCs, consider --all", cross!());
                panic!("empty list of IOCs")
            } else {
                debug!("{} {} selected", exclaim!(), "--all".bold().yellow());
                error!("{} --all not implemented yet, sorry.", cross!());
                panic!("--all not implemented yet.")
            };
        } // check if `all` --> get list from filesystem
        _ => panic!(),
    }
}

fn filter_duplicates(paths: Vec<String>) -> io::Result<Vec<String>> {
    let mut unique_paths: HashSet<PathBuf> = HashSet::new();
    let mut result: Vec<String> = Vec::new();

    for path in paths {
        let glob_paths: Vec<PathBuf> = glob(path.as_str())
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();

        for glob_path in glob_paths {
            let metadata = fs::metadata(&glob_path)?;
            if metadata.is_dir() {
                let abs_path = fs::canonicalize(&glob_path)?;
                if unique_paths.insert(abs_path.clone()) {
                    let abs_path_str = abs_path
                        .to_str()
                        .expect("Failed to convert path to string")
                        .to_owned();
                    result.push(abs_path_str);
                }
            }
        }
    }
    Ok(result)
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
