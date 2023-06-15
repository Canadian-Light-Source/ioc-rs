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
    use crate::test_utils::new_test_ioc;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_prep_stage_success() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();
        fs::create_dir_all(&test_ioc.stage)?;
        let test_file = test_ioc.stage.join("file1.txt");
        fs::write(&test_file, "dummy file")?;
        assert!(prep_stage(&test_ioc).is_ok());
        assert!(!test_file.exists());
        Ok(())
    }

    #[test]
    fn test_prep_stage_permission_denied_error() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();

        fs::create_dir_all(&test_ioc.stage)?;
        fs::write(test_ioc.stage.join("file1.txt"), "dummy file")?;
        let mut perms = fs::metadata(&test_ioc.stage).unwrap().permissions();
        if !perms.readonly() {
            perms.set_readonly(true);
            fs::set_permissions(&test_ioc.stage, perms).unwrap();
        }
        assert!(prep_stage(&test_ioc).is_err());
        Ok(())
    }

    #[test]
    fn test_ioc_stage_error() -> io::Result<()> {
        let settings = Settings::build("./tests/config/test_stage.toml").unwrap();

        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");

        // set the trap, create stage and make it readonly
        fs::create_dir_all(&stage_dir)?;
        // make target readonly --> can't create stage_root --> fail
        let mut perms = fs::metadata(&stage_dir).unwrap().permissions();
        if !perms.readonly() {
            perms.set_readonly(true);
            fs::set_permissions(&stage_dir, perms).unwrap();
        }
        // let the stage_root be a directory in the write only stage --> creating will fail
        let stage_root = stage_dir.join("no_write_permission");

        let test_ioc = IOC::new(Path::new("./tests/UTEST_IOC01"), stage_root, dest_dir).unwrap();
        ioc_stage(&None, Some(test_ioc.clone()), &settings);
        // the actual check
        assert!(!&test_ioc.stage.exists());
        Ok(())
    }

    #[test]
    fn test_stage_ioc_struct_success() -> io::Result<()> {
        let settings = Settings::build("./tests/config/test_stage.toml").unwrap();

        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");

        let test_ioc = IOC::new(Path::new("./tests/UTEST_IOC01"), stage_dir, dest_dir).unwrap();
        ioc_stage(&None, Some(test_ioc.clone()), &settings);
        // the actual check
        assert!(&test_ioc.stage.exists());
        Ok(())
    }

    #[test]
    fn test_stage_ioc_name_success() -> io::Result<()> {
        let settings = Settings::build("./tests/config/test_stage.toml").unwrap();
        let stage_root = settings.get::<String>("filesystem.stage").unwrap();

        ioc_stage(&Some("tests/UTEST_IOC01".to_string()), None, &settings);
        // the actual check
        let stage_dir = Path::new("./tests/tmp/stage/UTEST_IOC01");
        assert!(stage_dir.exists());
        assert!(fs::remove_dir_all(stage_root).is_ok());
        Ok(())
    }
}
