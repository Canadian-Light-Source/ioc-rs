use crate::log_macros::{cross, exclaim};
use colored::Colorize;
use config::{Config, ConfigError, File};
use log::{error, warn};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct IocConfig {
    pub port: u16,
    pub host: String,
    pub user: Option<String>,
    pub base_dir: Option<String>,
    pub name: Option<String>,
    pub procserv_opts: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub ioc: IocConfig,
}

impl Settings {
    pub fn build(config_file: &str) -> Result<Config, ConfigError> {
        let mut s = Config::builder()
            .set_default("ioc.user", "control2")
            .unwrap()
            .set_default("ioc.port", "0")
            .unwrap()
            .set_default("ioc.host", "localhost")
            .unwrap()
            .set_default("ioc.command", "iocsh")
            .unwrap()
            // local dev configuration
            .add_source(File::with_name(config_file).required(false))
            .build()?;
        // force hostname to lowercase
        s = rebuild("ioc.host".to_string(), check_hostname(&mut s)?, s.clone())?;
        Ok(s)
    }
}

fn check_hostname(conf: &mut Config) -> Result<String, ConfigError> {
    let mut hostname = conf.get_string("ioc.host").unwrap();
    if hostname.len() >= 16 {
        error!("{} hostname has {} characters.", cross!(), hostname.len());
        let e = ConfigError::Message("Hostname too long.".to_string());
        return Err(e);
    }
    if hostname.chars().any(char::is_uppercase) {
        hostname = hostname.to_lowercase();
        warn!(
            "{} IOC hostname was changed to lower case! --> '{}'",
            exclaim!(),
            hostname
        );
    };
    Ok(hostname)
}

fn rebuild(key: String, value: String, config: Config) -> Result<Config, ConfigError> {
    let s = Config::builder()
        .add_source(config)
        .set_override(key, value)
        .unwrap()
        .build()?;
    Ok(s)
}
