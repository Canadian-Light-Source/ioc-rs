use std::collections::HashSet;
use std::path::{Path, PathBuf};

use colored::Colorize;
use config::Config;
use glob::glob;
use log::{debug, error, info, trace};
use std::{env, fs, io};

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
    nodiff: &bool,
    force: &bool,
) -> io::Result<()> {
    let unique_iocs = check_ioc_list(iocs);
    let stage_root = settings.get::<String>("filesystem.stage").unwrap();
    let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
    let shellbox_root = settings.get::<String>("filesystem.shellbox").unwrap();
    let template_dir = settings.get::<String>("app.template_directory").unwrap();

    trace!("configuration ---------------------------");
    trace!("  stage:    {:?}", stage_root);
    trace!("  deploy:   {:?}", deploy_root);
    trace!("  templates:{:?}", template_dir);
    trace!("-----------------------------------------");

    let ioc_list = IOC::from_list(
        &unique_iocs,
        &stage_root,
        &deploy_root,
        &shellbox_root,
        &template_dir,
    );
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
        match stage::stage(ioc) {
            Ok(_) => {}
            Err(e) => {
                error!("{}, staging failed with: {}", cross!(), e);
                ioc_cleanup(ioc)?;
                remove_dir(Path::new(&stage_root))?;
                continue;
            }
        }

        if ioc.destination.exists() && !*nodiff {
            // hah, not nodiff, like a proper Bavarian :)
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
        let _ = ioc_shellbox(ioc);

        // deployment
        if !dryrun {
            // actual deployment run
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
            // dryrun
            info!("{} was chosen, no deployment", "--dryrun".yellow());
            match ioc_cleanup(ioc) {
                Ok(_) => {}
                Err(e) => error!(
                    "{} clean up failed for {} with error: {}",
                    cross!(),
                    &ioc.name,
                    e
                ),
            };
        }
        trace!("------------");
    }
    Ok(())
}

fn check_ioc_list(list: &Option<Vec<String>>) -> Vec<String> {
    match list {
        Some(l) => filter_duplicates(l.clone()).expect("unable to filter duplicates!"),
        None => {
            debug!("{} empty list of IOCs, using current_dir", exclaim!());
            filter_duplicates(vec![get_current_dir()]).expect("unable to filter duplicates!")
        }
    }
}

fn get_current_dir() -> String {
    if let Ok(current_dir) = env::current_dir() {
        current_dir.to_str().unwrap().to_string()
    } else {
        panic!("Failed to get current working directory")
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

fn ioc_cleanup(ioc: &IOC) -> io::Result<()> {
    trace!("cleaning up staging directory for {}", &ioc.name);
    remove_dir(&ioc.stage)?;
    info!("{} removed {:?}", tick!(), &ioc.stage);
    Ok(())
}

fn remove_dir(dir: impl AsRef<Path>) -> io::Result<()> {
    trace!("removing directory {}", dir.as_ref().to_str().unwrap());
    match fs::remove_dir_all(dir) {
        Ok(_) => {}
        Err(e) => error!("{} {}", cross!(), e),
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    // check if the first element of the returned vector is a directory.
    fn test_check_ioc_list_empty_list_all() {
        assert!(Path::new(check_ioc_list(&None).first().unwrap()).is_dir());
    }

    #[test]
    fn test_filter_duplicates() -> io::Result<()> {
        let input = vec!["foo", "bar"];

        let temp_dir = tempdir()?;
        for dir in &input {
            let full_path = temp_dir.path().join(dir);
            fs::create_dir_all(full_path)?
        }

        let mut expected: Vec<String> = input
            .iter()
            .map(|&p| temp_dir.path().join(p).to_str().unwrap().to_string())
            .collect();
        expected.sort();

        let glob_test = temp_dir.path().join("*").to_str().unwrap().to_string();
        let res = filter_duplicates(vec![glob_test.clone()])?;
        assert_eq!(res, expected);

        let foo_test = temp_dir.path().join("foo").to_str().unwrap().to_string();
        let res = filter_duplicates(vec![glob_test.clone(), foo_test.clone()])?;
        assert_eq!(res, expected);

        let bar_test = temp_dir.path().join("bar").to_str().unwrap().to_string();
        let res = filter_duplicates(vec![glob_test.clone(), bar_test.clone()])?;
        assert_eq!(res, expected);

        let res = filter_duplicates(vec![bar_test.clone(), foo_test.clone()])?;
        assert_eq!(res, expected);

        let mut res = filter_duplicates(vec![foo_test])?;
        assert_eq!(res.pop().unwrap(), expected.pop().unwrap());

        let mut res = filter_duplicates(vec![bar_test])?;
        assert_eq!(res.pop().unwrap(), expected.pop().unwrap());

        Ok(())
    }
}
