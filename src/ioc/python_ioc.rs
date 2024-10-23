use std::process::Command;
use std::path::Path;

use std::io::{self, Error};
use glob::glob;

// logging
use colored::Colorize;
use log::{error, trace, warn, info};
use crate::log_macros::{cross, tick,exclaim};

/// Customized Conda environment folder
pub const CONDA_ENV_DIR: &str = "env";

/// Customized Conda environment yaml description.
pub const CONDA_ENV_CFG: &str = "config.yaml";


// search for python file
pub fn search_python(dir: &str) -> io::Result<()> {
    let pattern = format!("{}/**/*.py", dir.to_lowercase());

    // Use glob to find Python files and handle errors
    let entries = glob(&pattern)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?; // Convert glob error to io::Error

    for entry in entries {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    trace!("{} Found at least one Python file: {}", tick!(), path.display());
                    return Ok(());
                }
            }
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }

    warn!("{} No Python files found.", exclaim!());
    Err(io::Error::new(io::ErrorKind::NotFound, "No Python files found"))
}



// function to create customized conda environment if case config.yaml is found
pub fn create_conda_env( destination: impl AsRef<Path> ) -> io::Result<()> {
    
    let env_dir = destination.as_ref().join(CONDA_ENV_DIR); // self.destination.join("env");
    let cfg_path = CONDA_ENV_CFG;

    // Parse the YAML config inside this function
    match Path::new(cfg_path).exists() {
        true => {
            trace!("{} Found env config: {}", tick!(), cfg_path);
        },
        false =>{
            info!("{} Conda env config not found. IOC will use default", tick!());
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Conda env not found"));
        }
    };

    // Check if the destination folder exists, create it if not
    match Path::new(&env_dir).exists() {
        true => {
            trace!("{} Found env config: {}", tick!(), cfg_path);
        },
        false =>{
            info!("{} Destination folder does not exist. Creating: '{}'", tick!(),env_dir.display());
            match std::fs::create_dir_all(env_dir.as_path()) {
                Ok(_) => {
                    info!("{} Destination folder created", tick!());
                },
                Err(e) => {
                    warn!("{} Could not create conda env. IOC will use default one, Err: {}", exclaim!(),e);
                    return Err(e);
                }
            };
        }
    };
    
    // Execute the Bash script
    match Command::new("/opt/tools/conda/bin/create_conda_env.sh")
    .arg(cfg_path)            // pass config.yaml as argument 1
    .arg(env_dir.to_str().unwrap().to_string())     // pass destination folder as argument 2
    .status() {
        Ok(out) => {
            info!("{} Output ok : {}", tick!(), out);
            out
        },
        Err(e) => {            
            error!("{} Error output command 'conda create': {}", cross!(),e);
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Failed conda create"));
        }
    };

    Ok(())
}
