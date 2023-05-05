use std::env;
use std::io;
use std::path::{Path, PathBuf};

// logging
use colored::Colorize;
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
            }),
            false => Err("Could not find source of IOC."),
        }
    }

    pub fn from_list(
        list: &[String],
        stage_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>,
    ) -> Vec<Self> {
        debug!("collecting iocs ...");
        list.iter()
            .map(|name| {
                let work_dir = env::current_dir().unwrap().join(name);
                trace!("working dir: {:?}", work_dir);
                // TODO: `match` this to create pleasing Error log
                IOC::new(&work_dir, &stage_root, &destination_root).expect("from_list failed")
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
