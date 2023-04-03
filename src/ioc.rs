use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use tera::{Context, Error, Tera};
use users::get_current_username;

// for checksum
use blake2::{Blake2s256, Digest};
use file_hashing::get_hash_folder;

// logging
use colored::Colorize;
use log::{debug, error, info, trace};

use crate::diff::get_patch;
use crate::log_macros::tick;
use crate::PackageData;

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
        render::render_startup(&self, template_dir)?;
        debug!("{} staging of {:?} complete.", tick!(), self.name);
        Ok(())
    }

    pub fn hash_ioc(&self) -> std::io::Result<()> {
        let hash = calc_directory_hash(&self.stage);
        trace!("hash: {:?}", hash);
        fs::create_dir_all(&self.data)?;
        write_file(&self.hash_file, hash)?;
        debug!(
            "{} hash_file {:?} written.",
            tick!(),
            &self.hash_file.as_path()
        );
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
        self.hash_ioc()?;
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

    /// check whether destination has been tempered with
    pub fn check_hash(&self) -> Result<String, String> {
        // destination doesn't exist yet, that's fine
        if !self.destination.exists() {
            return Ok("destination does not yet exist. No hash expected.".to_string());
        }
        let mut hash = String::from("");
        if let Ok(lines) = read_lines(&self.hash_file) {
            if let Ok(stored_hash) = lines.last().unwrap() {
                hash = stored_hash;
            };
        }

        let valid_hash = match hash == calc_directory_hash(&self.destination) {
            false => return Err("hashes do not match".to_string()),
            true => hash,
        };
        Ok(valid_hash)
    }
}

fn write_file(file_name: impl AsRef<Path>, content: String) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn calc_directory_hash(dir: impl AsRef<Path>) -> String {
    let mut hash = Blake2s256::new();
    let directory = dir.as_ref().to_str().unwrap();
    get_hash_folder(directory, &mut hash, 1, |_| {}).unwrap()
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
            let patch = get_patch(entry.path(), destination.as_ref().join(entry.file_name()))?;
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
