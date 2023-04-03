use config::{Config, ConfigError, File};
use serde_derive::Deserialize;
// use std::env;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Filesystem {
    pub stage: String,
    pub deploy: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct App {
    pub template_directory: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub debug: bool,
    pub filesystem: Filesystem,
    pub app: App,
}

impl Settings {
    pub fn build(config_file: &str) -> Result<Config, ConfigError> {
        let s = Config::builder()
            .set_default("debug", false)
            .unwrap()
            .set_default("filesystem.stage", "./stage")
            .unwrap()
            .set_default("filesystem.deploy", "./ioc/delpoy")
            .unwrap()
            .set_default("app.templates", "/opt/apps/ioc/templates/*.tera")
            .unwrap()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("/opt/apps/ioc/config/default").required(true))
            // local dev configuration
            .add_source(File::with_name("config/dev").required(false))
            // Add in the config file from cli
            .add_source(File::with_name(config_file).required(!config_file.is_empty()))
            .build()?;
        Ok(s)
    }
}
