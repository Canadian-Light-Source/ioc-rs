use crate::log_macros::{cross, tick};
use colored::Colorize;
use config::{Config, ConfigError, File};
use log::{debug, error, trace};
use serde_derive::Deserialize;
use std::{
    env, io,
    path::{Path, PathBuf},
};

const IOC_CONFIG_NAME: &str = ".iocrc";
const IOC_CONFIG_PATH: &str = "IOC_CONFIG_PATH";
const IOC_DIR: &str = "ios";
const CONFIG_DIR: &str = ".config";
const HOME: &str = "HOME";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Filesystem {
    pub stage: String,
    pub deploy: String,
    pub shellbox: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct App {
    pub template_directory: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub filesystem: Filesystem,
    pub app: App,
}

/// Reads the config file into a `String` if there is one. When `None` is provided then the config
/// is looked for in the following locations in order:
///
/// - `$IOC_CONFIG_PATH`
/// - `$XDG_CONFIG_HOME/ioc/.iocrc`
/// - `$XDG_CONFIG_HOME/.iocrc`
/// - `$HOME/.config/ioc/.iocrc`
/// - `$HOME/.iocrc`
pub fn cfg_path_to_string<T: AsRef<Path>>(path: Option<T>) -> Option<String> {
    path.map(get_path_if_is_file)
        .and_then(Result::ok)
        .or_else(config_from_config_path)
        .or_else(config_from_xdg_path)
        .or_else(config_from_home)
}

fn get_path_if_is_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    match path.as_ref().is_file() {
        true => Ok(path.as_ref().to_str().unwrap().to_string()),
        false => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
    }
}
/// Try to read in config from `IOC_CONFIG_PATH`.
fn config_from_config_path() -> Option<String> {
    env::var_os(IOC_CONFIG_PATH)
        .map(PathBuf::from)
        .map(get_path_if_is_file)
        .and_then(Result::ok)
}

fn config_from_xdg_path() -> Option<String> {
    let xdg_config = env::var_os(XDG_CONFIG_HOME).map(PathBuf::from)?;
    let config_path = xdg_config.join(IOC_DIR).join(IOC_CONFIG_NAME);

    get_path_if_is_file(config_path).ok().or_else(|| {
        let config_path = xdg_config.join(IOC_CONFIG_NAME);
        get_path_if_is_file(config_path).ok()
    })
}

fn config_from_home() -> Option<String> {
    let home = env::var_os(HOME).map(PathBuf::from)?;
    let config_path = home.join(CONFIG_DIR).join(IOC_DIR).join(IOC_CONFIG_NAME);

    get_path_if_is_file(config_path).ok().or_else(|| {
        let config_path = home.join(IOC_CONFIG_NAME);
        get_path_if_is_file(config_path).ok()
    })
}

impl Settings {
    pub fn build(config_file: &str) -> Result<Config, ConfigError> {
        let cfg_file = match cfg_path_to_string(Some(Path::new(config_file))) {
            Some(p) => {
                debug!("{} found config file: {}", tick!(), p);
                p
            }
            None => {
                error!("{} {}", cross!(), "missing config file".red());
                panic!("config file is mandatory")
            }
        };
        // println!("====> {:?}", cfg_file);
        // read_config_to_string(Some(Path::new("")));
        let s = Config::builder()
            .add_source(File::with_name(cfg_file.as_str()).required(true))
            .build()?;
        trace!("{} {:?}", tick!(), s);
        Ok(s)
    }
}
