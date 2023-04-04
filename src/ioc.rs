use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// logging
use colored::Colorize;
use log::{debug, info, trace};

use crate::diff::get_patch;
use crate::hash_ioc;
use crate::log_macros::tick;
use crate::render;

/// IOC structure
#[derive(Debug)]
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
}

/// IOC structure implementation
impl IOC {
    /// Creates a new IOC structure
    /// shold fail if source does not contain at least a 'startup.iocsh'
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
        // check source exists
        match source.as_ref().is_dir() {
            true => Ok(IOC {
                name,
                source: source.as_ref().to_path_buf(),
                stage,
                data,
                hash_file,
                destination,
            }),
            false => Err("Could not find source of IOC."),
        }
    }

    pub fn stage(&self, template_dir: &str) -> std::io::Result<()> {
        trace!("staging {}", self.name.blue());
        if self.stage.exists() {
            remove_dir_contents(&self.stage)?; // prep stage directory
            trace!("{} {:?} removed", tick!(), &self.stage.as_path());
        }
        copy_recursively(&self.source, &self.stage)?;
        trace!(
            "{} copied {:?} -> {:?}",
            tick!(),
            &self.source.as_path(),
            &self.stage.as_path()
        );
        render::render_startup(self, template_dir)?;
        debug!("{} staging of {:?} complete.", tick!(), self.name);
        Ok(())
    }

    pub fn diff_ioc(&self) -> std::io::Result<()> {
        trace!("diff for {}", self.name.blue());
        diff_recursively(&self.stage, &self.destination)?;
        Ok(())
    }

    pub fn deploy(&self) -> std::io::Result<()> {
        trace!("deploying {}", self.name.blue());
        if self.destination.exists() {
            remove_dir_contents(&self.destination)?; // prep deploy directory
            trace!("{} removed {:?}", tick!(), &self.destination);
        }
        hash_ioc::hash_ioc(self)?;
        copy_recursively(&self.stage, &self.destination)?;
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

/// Copy files from source to destination recursively.
fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name().into_string().unwrap().starts_with('.') {
            continue;
        }
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            if entry.file_name().into_string().unwrap() == "cfg" {
                copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                copy_recursively(entry.path(), destination.as_ref())?; // flatten the structure
            }
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn diff_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name().into_string().unwrap().starts_with('.') {
            continue;
        }
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            if entry.file_name().into_string().unwrap() == "cfg" {
                diff_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                diff_recursively(entry.path(), destination.as_ref())?; // flatten the structure
            }
        } else {
            let patch = get_patch(destination.as_ref().join(entry.file_name()), entry.path())?;
            if patch.lines().count() > 3 {
                info!("===========================================================");
                info!("--- original: {}", entry.path().to_str().unwrap());
                info!(
                    "+++ modified: {}",
                    destination
                        .as_ref()
                        .join(entry.file_name())
                        .to_str()
                        .unwrap()
                );
                info!("DIFF:\n{}", patch);
                info!("===========================================================");
            }
        }
    }
    Ok(())
}

fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            remove_dir_contents(&path)?;
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
