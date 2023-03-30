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
    pub fn new(config_file: &str) -> Result<Config, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("/opt/apps/ioc/config/default").required(true))
            // local dev configuration
            .add_source(File::with_name("config/dev").required(false))
            // Add in the config file from cli
            .add_source(File::with_name(config_file).required(!config_file.is_empty()))
            // Default to 'development' env
            // Note that this file is _optional_
            // .add_source(
            //     File::with_name(&format!("examples/hierarchical-env/config/{}", run_mode))
            //         .required(false),
            // )
            // // Add in a local configuration file
            // // This file shouldn't be checked in to git
            // // Add in settings from the environment (with a prefix of APP)
            // // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            // .add_source(Environment::with_prefix("app"))
            // // You may also programmatically change settings
            // .set_override("database.url", "postgres://")?
            .build()?;

        // // Now that we're done, let's access our configuration
        // println!("debug:  {:?}", s.get_bool("debug"));
        // println!("stage:  {:?}", s.get::<String>("filesystem.stage"));
        // println!("deploy: {:?}", s.get::<String>("filesystem.deploy"));

        // // You can deserialize (and thus freeze) the entire configuration as
        // // s.try_deserialize()
        Ok(s)
    }
}
