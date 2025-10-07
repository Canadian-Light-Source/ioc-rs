use crate::log_macros::{cross, tick};
use colored::Colorize;
use config::{Config, ConfigError, File};
use log::{debug, error, trace};
use serde_derive::Deserialize;
use std::{
    env, io,
    path::{Path, PathBuf},
};
use tera::Tera;

const IOC_CONFIG_NAME: &str = "ioc.toml";
const IOC_CONFIG_FILE: &str = "IOC_CONFIG_FILE";
const IOC_DIR: &str = "ioc";
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

/// Returns the config file path as `String` if there is one. When `None` is provided then the config
/// is looked for in the following locations in order:
///
/// - `$IOC_CONFIG_FILE`
/// - `$XDG_CONFIG_HOME/ioc/.ioc.toml`
/// - `$XDG_CONFIG_HOME/.ioc.toml`
/// - `$HOME/.config/ioc/.ioc.toml`
/// - `$HOME/.ioc.toml`
pub fn cfg_path_to_string<T: AsRef<Path>>(path: Option<T>) -> Option<String> {
    path.map(get_path_if_is_file)
        .and_then(Result::ok)
        .or_else(config_from_config_path)
        .or_else(config_from_xdg_path)
        .or_else(config_from_home)
}

/// checks if the path is a file and returns the path as `String` if so, else return an error.
fn get_path_if_is_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    debug!("searching for ioc-rs config file in: {:?}", path.as_ref());
    match path.as_ref().is_file() {
        true => Ok(path.as_ref().to_str().unwrap().to_string()),
        false => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
    }
}

/// Try to find a config in `IOC_CONFIG_FILE`.
fn config_from_config_path() -> Option<String> {
    env::var_os(IOC_CONFIG_FILE)
        .map(PathBuf::from)
        .map(get_path_if_is_file)
        .and_then(Result::ok)
}

/// Try to find a config in `XDG_CONFIG_HOME`.
fn config_from_xdg_path() -> Option<String> {
    let xdg_config = env::var_os(XDG_CONFIG_HOME).map(PathBuf::from)?;
    let config_path = xdg_config.join(IOC_DIR).join(IOC_CONFIG_NAME);

    get_path_if_is_file(config_path).ok().or_else(|| {
        let config_path = xdg_config.join(IOC_CONFIG_NAME);
        get_path_if_is_file(config_path).ok()
    })
}

/// Try to find a config in `HOME`.
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

        let s = Config::builder()
            .add_source(File::with_name(cfg_file.as_str()).required(true))
            .build()?;
        trace!("{} {:?}", tick!(), s);
        Ok(s)
    }

    pub fn verify(config: &Config) -> Result<(), ConfigError> {
        let template_dir = config.get::<String>("app.template_directory")?;
        let tera = match Tera::new(&template_dir) {
            Ok(t) => t,
            Err(e) => {
                error!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        if tera.get_template_names().collect::<Vec<_>>().is_empty() {
            return Err(ConfigError::Message(
                "The specified template path does not contain valid templates".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn get_path_if_is_file_success() -> io::Result<()> {
        // Set up a temporary directory to use as the stage and destination.
        let temp_dir = tempdir()?;
        std::fs::write(temp_dir.path().join("file1.txt"), "")?;

        assert!(get_path_if_is_file(temp_dir.path().join("file1.txt")).is_ok());
        Ok(())
    }

    #[test]
    fn get_path_if_is_file_error() {
        assert!(get_path_if_is_file("file1.txt").is_err());
    }

    #[test]
    fn config_from_config_path_none() {
        assert_eq!(config_from_config_path(), None);
    }

    #[test]
    fn config_from_xdg_path_none() {
        assert_eq!(config_from_xdg_path(), None);
    }

    #[test]
    #[serial]
    fn config_from_home_none() {
        // Save the original HOME value
        let original_home = env::var_os(HOME);

        // Set HOME to a non-existent directory
        let temp_dir = tempfile::Builder::new().tempdir().unwrap();
        let non_existent_dir = temp_dir.path().join("non_existent");
        env::set_var(HOME, &non_existent_dir);

        let result = config_from_home();

        // Restore the original HOME value
        match original_home {
            Some(home) => env::set_var(HOME, home),
            None => env::remove_var(HOME),
        }

        assert_eq!(result, None);
    }

    #[test]
    #[serial]
    fn config_from_config_path_success() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let cfg_file = temp_dir.path().join(IOC_CONFIG_NAME);
        std::fs::write(&cfg_file, "")?;
        env::set_var("IOC_CONFIG_FILE", cfg_file.as_os_str());

        assert_eq!(
            config_from_config_path(),
            Some(cfg_file.to_str().unwrap().to_string())
        );
        Ok(())
    }

    #[test]
    #[serial]
    fn config_from_xdg_path_root_success() -> io::Result<()> {
        let temp_dir = tempdir()?;

        let cfg_file = temp_dir.path().join(IOC_CONFIG_NAME);
        std::fs::write(&cfg_file, "")?;
        env::set_var(XDG_CONFIG_HOME, temp_dir.path().as_os_str());

        assert_eq!(
            config_from_xdg_path(),
            Some(cfg_file.to_str().unwrap().to_string())
        );
        Ok(())
    }

    #[test]
    #[serial]
    fn config_from_xdg_path_sub_success() -> io::Result<()> {
        // Save original environment
        let original_xdg = env::var_os(XDG_CONFIG_HOME);

        let temp_dir = tempdir()?;
        env::set_var(XDG_CONFIG_HOME, temp_dir.path().as_os_str());
        let ioc_dir = temp_dir.path().join(IOC_DIR);
        std::fs::create_dir_all(&ioc_dir)?;

        let cfg_file = ioc_dir.join(IOC_CONFIG_NAME);
        std::fs::write(&cfg_file, "")?;

        let result = config_from_xdg_path();

        // Restore original environment
        match original_xdg {
            Some(xdg) => env::set_var(XDG_CONFIG_HOME, xdg),
            None => env::remove_var(XDG_CONFIG_HOME),
        }

        assert_eq!(result, Some(cfg_file.to_str().unwrap().to_string()));
        Ok(())
    }

    #[test]
    #[serial]
    fn config_from_home_root_success() -> io::Result<()> {
        let temp_dir = tempdir()?;

        let cfg_file = temp_dir.path().join(IOC_CONFIG_NAME);
        std::fs::write(&cfg_file, "")?;
        env::set_var(HOME, temp_dir.path().as_os_str());

        assert_eq!(
            config_from_home(),
            Some(cfg_file.to_str().unwrap().to_string())
        );
        Ok(())
    }

    #[test]
    #[serial]
    fn config_from_home_config_success() -> io::Result<()> {
        // Save original environment
        let original_home = env::var_os(HOME);
        let original_xdg = env::var_os(XDG_CONFIG_HOME);

        let temp_dir = tempdir()?;
        env::set_var(HOME, temp_dir.path().as_os_str());
        // Clear XDG to avoid interference
        env::remove_var(XDG_CONFIG_HOME);

        let config_dir = temp_dir.path().join(CONFIG_DIR).join(IOC_DIR);
        std::fs::create_dir_all(&config_dir)?;

        let cfg_file = config_dir.join(IOC_CONFIG_NAME);
        std::fs::write(&cfg_file, "")?;

        let result = config_from_home();

        // Restore original environment
        match original_home {
            Some(home) => env::set_var(HOME, home),
            None => env::remove_var(HOME),
        }
        match original_xdg {
            Some(xdg) => env::set_var(XDG_CONFIG_HOME, xdg),
            None => env::remove_var(XDG_CONFIG_HOME),
        }

        assert_eq!(result, Some(cfg_file.to_str().unwrap().to_string()));
        Ok(())
    }
}
