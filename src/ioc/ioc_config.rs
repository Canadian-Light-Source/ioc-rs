use config::{Config, ConfigError, File};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct IocConfig {
    pub port: u32,
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
        let s = Config::builder()
            .set_default("ioc.user", "control")
            .unwrap()
            // local dev configuration
            .add_source(File::with_name(config_file).required(true))
            .build()?;
        Ok(s)
    }
}
