use std::io;

use colored::Colorize;
use log::{debug, error, info, warn};

use crate::log_macros::{cross, exclaim};
use crate::origin::Origin;
use crate::{file_system, ioc::IOC, log_macros::tick};
pub mod render;

pub fn stage(ioc: &IOC) -> io::Result<()> {
    info!("staging {}", ioc.name.blue());

    match prep_stage(ioc) {
        Ok(_) => debug!("{} stage prepared.", tick!()),
        Err(e) => {
            error!(
                "{} failed to prepare stage with: {}",
                cross!(),
                e.to_string().red()
            );
            return Err(e);
        }
    }

    match file_system::copy_recursively(&ioc.source, &ioc.stage) {
        Ok(_) => debug!(
            "{} copied {:?} -> {:?}",
            tick!(),
            &ioc.source.as_path(),
            &ioc.stage.as_path()
        ),
        Err(e) => {
            error!(
                "{} failed to copy files with: {}",
                cross!(),
                e.to_string().red()
            );
            return Err(e);
        }
    }

    match Origin::new(&ioc.source).write_origin_file(&ioc.stage) {
        Ok(_) => debug!("{} ORIGIN file written.", tick!()),
        Err(e) => {
            error!(
                "{} failed to write ORIGIN file with: {}",
                cross!(),
                e.to_string().red()
            );
            return Err(e);
        }
    }

    match render::render_startup(ioc, ioc.templates.as_os_str().to_str().unwrap()) {
        Ok(_) => debug!("{}, startup script rendered.", tick!()),
        Err(e) => {
            error!(
                "{} failed to render startup script with: {}",
                cross!(),
                e.to_string().red()
            );
            return Err(e);
        }
    }

    // TODO: Add shellbox here?
    info!(
        "{} staging of {:?} in {:?} complete.",
        tick!(),
        ioc.name,
        ioc.stage
    );
    Ok(())
}

fn prep_stage(ioc: &IOC) -> io::Result<()> {
    if ioc.stage.exists() {
        warn!(
            "{} stage directory already exists, attempting to fix that.",
            exclaim!()
        );
        match file_system::remove_dir_contents(&ioc.stage) {
            Ok(_) => debug!("{} {:?} removed", tick!(), &ioc.stage.as_path()),
            Err(e) => {
                error!(
                    "{} error while clearing stage directory: {}",
                    cross!(),
                    e.to_string().red()
                );
                return Err(e);
            }
        };
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
        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        let shellbox_root = temp_dir.path().join("shellbox");
        let template_dir = temp_dir.path().join("templates");

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

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            stage_root,
            dest_dir,
            shellbox_root,
            template_dir,
        )
        .unwrap();
        // the actual checks
        assert!(stage(&test_ioc).is_err());
        assert!(!&test_ioc.stage.exists());
        Ok(())
    }

    #[test]
    fn test_stage_ioc_struct_success() -> io::Result<()> {
        let settings = Settings::build("./tests/config/test_stage.toml").unwrap();
        let template_dir = settings.get::<String>("app.template_directory").unwrap();

        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        let shellbox_root = temp_dir.path().join("shellbox");

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            stage_dir,
            dest_dir,
            shellbox_root,
            template_dir,
        )
        .unwrap();
        // the actual checks
        assert!(stage(&test_ioc).is_ok());
        assert!(&test_ioc.stage.exists());
        Ok(())
    }
}
