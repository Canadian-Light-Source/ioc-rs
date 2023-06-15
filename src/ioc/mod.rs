use std::env;
use std::io;
use std::path::{Path, PathBuf};

// logging
use colored::Colorize;
use config::Config;
use log::{debug, error, trace};

use crate::{
    file_system,
    log_macros::{cross, tick},
};

mod diff;
pub mod hash_ioc;
mod ioc_config;
pub mod render;

/// IOC structure
#[derive(Debug, Clone)]
pub struct IOC {
    /// name of the IOC
    pub name: String,
    /// source of the IOC definition
    pub source: PathBuf,
    /// staging directory
    pub stage: PathBuf,
    /// data directory for checksum
    pub data: PathBuf,
    /// hash file name
    pub hash_file: PathBuf,
    /// deploy directory for IOC
    pub destination: PathBuf,
    /// configuration
    pub config: ioc_config::Settings,
    /// template directory
    pub templates: PathBuf,
}

/// IOC structure implementation
impl IOC {
    /// Creates a new IOC structure
    /// should fail if source does not contain at least a 'startup.iocsh'
    // TODO: implement pre-check
    pub fn new(
        // name: &String,
        source: impl AsRef<Path>,
        stage_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>,
        template_root: impl AsRef<Path>,
    ) -> Result<IOC, &'static str> {
        let name = source
            .as_ref()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let stage = stage_root.as_ref().join(&name);
        let destination = destination_root.as_ref().join(&name);
        let data = destination_root.as_ref().join("data").join(&name);
        let hash_file = data.join("hash");
        let config = match ioc_config::Settings::build(
            source
                .as_ref()
                .to_path_buf()
                .join("config")
                .to_str()
                .unwrap(),
        ) {
            Ok(s) => s.try_deserialize().unwrap(),
            Err(e) => {
                error!("{} fatal error in IOC config: {}", cross!(), e);
                panic!("{:?}", e);
            }
        };
        // check source exists
        match source.as_ref().is_dir() {
            true => Ok(IOC {
                name,
                source: source.as_ref().to_path_buf(),
                stage,
                data,
                hash_file,
                destination,
                config,
                templates: template_root.as_ref().to_path_buf(),
            }),
            false => {
                println!(
                    "{} IOC source not found. {}",
                    cross!(),
                    source.as_ref().to_string_lossy()
                );
                Err("Could not find source of IOC.")
            }
        }
    }

    pub fn new_with_settings(source: impl AsRef<Path>, settings: &Config) -> Self {
        let stage_root = settings.get::<String>("filesystem.stage").unwrap();
        let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
        let template_dir = settings.get::<String>("app.template_directory").unwrap();
        Self::new(&source, &stage_root, &deploy_root, &template_dir).expect("from_list failed")
    }

    pub fn from_list(
        list: &[String],
        stage_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>,
        template_dir: impl AsRef<Path>,
    ) -> Vec<Self> {
        debug!("collecting iocs ...");
        list.iter()
            .map(|name| {
                let work_dir = env::current_dir().unwrap().join(name);
                trace!("working dir: {:?}", work_dir);
                // TODO: `match` this to create pleasing Error log
                IOC::new(&work_dir, &stage_root, &destination_root, &template_dir)
                    .expect("from_list failed")
            })
            .collect()
    }

    pub fn diff_ioc(&self) -> io::Result<()> {
        trace!("diff for {}", self.name.blue());
        diff::diff_recursively(&self.stage, &self.destination)?;
        Ok(())
    }

    pub fn deploy(&self) -> io::Result<()> {
        trace!("deploying {}", self.name.blue());
        if self.destination.exists() {
            file_system::remove_dir_contents(&self.destination)?; // prep deploy directory
            trace!("{} removed {:?}", tick!(), &self.destination);
        }
        hash_ioc::hash_ioc(self)?;
        file_system::copy_recursively(&self.stage, &self.destination)?;
        trace!(
            "{} copied {:?} -> {:?}",
            tick!(),
            &self.stage.as_path(),
            &self.destination.as_path()
        );
        debug!(
            "{} deployment of {:?} to {:?} complete.",
            tick!(),
            self.name,
            &self.destination.as_path()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::settings::Settings;
    use crate::stage;
    use tempfile::tempdir;

    #[test]
    fn diff_ioc_success() -> io::Result<()> {
        // Set up a temporary directory to use as the stage and destination.
        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        let template_dir = temp_dir.path().join("templates");

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            stage_dir,
            dest_dir,
            template_dir,
        )
        .unwrap();

        std::fs::create_dir_all(&test_ioc.stage)?;
        std::fs::create_dir_all(&test_ioc.destination)?;
        std::fs::write(test_ioc.stage.join("file1.txt"), "file1 contents")?;
        std::fs::write(test_ioc.destination.join("file1.txt"), "file1 contents mod")?;

        assert!(test_ioc.diff_ioc().is_ok());
        Ok(())
    }

    #[test]
    fn test_from_list_success() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let source_dir = temp_dir.path().join("source");
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        let template_dir = temp_dir.path().join("templates");
        let ioc_names = vec!["foo".to_string(), "bar".to_string()];
        let mut ioc_list: Vec<String> = Vec::new();
        for ioc in ioc_names.clone() {
            let path = &source_dir.join(ioc);
            std::fs::create_dir_all(path)?;
            ioc_list.extend_from_slice(&[path.to_str().unwrap().to_string()]);
        }
        let iocs = IOC::from_list(&ioc_list, stage_dir, dest_dir, template_dir);
        iocs.iter()
            .enumerate()
            .for_each(|(n, i)| assert_eq!(i.name, ioc_names[n]));
        Ok(())
    }

    #[test]
    fn test_ioc_deploy_success() -> io::Result<()> {
        let settings = Settings::build("tests/config/test_deploy.toml").unwrap();
        let template_dir = settings.get::<String>("app.template_directory").unwrap();

        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            stage_dir,
            dest_dir,
            template_dir,
        )
        .unwrap();
        assert!(stage::stage(&test_ioc).is_ok());
        assert!(test_ioc.deploy().is_ok());
        Ok(())
    }
}
