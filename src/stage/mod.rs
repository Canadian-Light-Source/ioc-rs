use std::{env, io};

use colored::Colorize;
use config::Config;
use log::{debug, error, info, trace};

use crate::{
    file_system,
    ioc::render,
    ioc::IOC,
    log_macros::{cross, tick},
};

pub fn ioc_stage(ioc_name: &Option<String>, ioc_struct: Option<IOC>, settings: &Config) {
    let stage_root = settings.get::<String>("filesystem.stage").unwrap();
    let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
    let template_dir = settings.get::<String>("app.template_directory").unwrap();

    trace!("configuration ---------------------------");
    trace!("  stage:    {:?}", stage_root);
    trace!("  deploy:   {:?}", deploy_root);
    trace!("  templates:{:?}", template_dir);
    trace!("-----------------------------------------");

    let ioc = match ioc_name {
        Some(name) => {
            let work_dir = env::current_dir().unwrap().join(name);
            trace!("working dir: {:?}", work_dir);
            IOC::new(&work_dir, &stage_root, &deploy_root).unwrap()
        }
        None => ioc_struct.unwrap(),
    };

    trace!("staging {}", ioc.name.blue().bold());
    match stage(&ioc, template_dir.as_str()) {
        Ok(_) => info!("{} staged {}", tick!(), ioc.name.blue()),
        Err(e) => error!(
            "{} staging of {} failed with: {}",
            cross!(),
            ioc.name.red().bold(),
            e
        ),
    }
}

fn stage(ioc: &IOC, template_dir: &str) -> io::Result<()> {
    trace!("staging {}", ioc.name.blue());
    prep_stage(ioc)?;
    file_system::copy_recursively(&ioc.source, &ioc.stage)?;
    trace!(
        "{} copied {:?} -> {:?}",
        tick!(),
        &ioc.source.as_path(),
        &ioc.stage.as_path()
    );
    render::render_startup(ioc, template_dir)?;
    // TODO: Add shellbox here?
    debug!("{} staging of {:?} complete.", tick!(), ioc.name);
    Ok(())
}

fn prep_stage(ioc: &IOC) -> io::Result<()> {
    if ioc.stage.exists() {
        file_system::remove_dir_contents(&ioc.stage)?; // prep stage directory
        trace!("{} {:?} removed", tick!(), &ioc.stage.as_path());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::settings::Settings;
    use std::fs;
    use std::path::Path;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_permission_denied_prep_stage() {
        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            Path::new("./tests/tmp/prep_stage/"),
            Path::new("./tests/tmp/deploy/ioc/"),
        )
        .unwrap();
        let result = prep_stage(&test_ioc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stage_fail_ioc() {
        let settings = Settings::build("tests/config/test_stage.toml").unwrap();
        let stage_root = Path::new("./tests/tmp/stage_test_fail/");

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            Path::new("./tests/tmp/stage_test_fail/no_write_permission"),
            Path::new("./tests/tmp/deploy/ioc/"),
        )
        .unwrap();
        // make target readonly --> can't create stage_root --> fail
        let mut perms = fs::metadata(stage_root).unwrap().permissions();
        if !perms.readonly() {
            perms.set_readonly(true);
            fs::set_permissions(stage_root, perms).unwrap();
        }
        ioc_stage(&None, Some(test_ioc.clone()), &settings);
        // the actual check
        assert!(!&test_ioc.stage.exists());
    }

    #[test]
    fn test_stage_ioc() {
        let settings = Settings::build("tests/config/test_stage.toml").unwrap();

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            Path::new("./tests/tmp/stage_test1/"),
            Path::new("./tests/tmp/deploy/ioc/"),
        )
        .unwrap();
        ioc_stage(&None, Some(test_ioc.clone()), &settings);
        // the actual check
        assert!(&test_ioc.stage.exists());
        assert!(fs::remove_dir_all(&test_ioc.stage).is_ok());
    }

    #[test]
    fn test_stage_ioc2() {
        let settings = Settings::build("tests/config/test_stage.toml").unwrap();
        let stage_root = settings.get::<String>("filesystem.stage").unwrap();

        ioc_stage(&Some("tests/UTEST_IOC01".to_string()), None, &settings);
        // the actual check
        let stage_dir = Path::new("./tests/tmp/stage/UTEST_IOC01");
        assert!(stage_dir.exists());
        assert!(fs::remove_dir_all(stage_root).is_ok());
    }
}
