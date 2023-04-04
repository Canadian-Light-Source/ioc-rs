use std::env;

use colored::Colorize;
use config::Config;
use log::{error, info, trace};

use crate::{
    ioc::IOC,
    log_macros::{cross, tick},
};

pub fn ioc_stage(ioc_name: &Option<String>, ioc_struct: Option<IOC>, settings: &Config) {
    let stage_root = settings.get::<String>("filesystem.stage").unwrap();
    let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
    let template_dir = settings.get::<String>("app.template_directory").unwrap();

    trace!("configuration ---------------------------");
    trace!("  stage:    {:?}", stage_root);
    trace!("  deploy:   {:?}", deploy_root);
    trace!("  templates:{:?}", template_dir);
    trace!("-----------------------------------------");

    let ioc = match ioc_name {
        Some(name) => {
            let work_dir = env::current_dir().unwrap().join(name);
            trace!("working dir: {:?}", work_dir);
            IOC::new(&work_dir, &stage_root, &deploy_root).unwrap()
        }
        None => ioc_struct.unwrap(),
    };

    trace!("staging {}", ioc.name.blue().bold());
    match ioc.stage(template_dir.as_str()) {
        Ok(_) => info!("{} staged {}", tick!(), ioc.name.blue()),
        Err(e) => error!(
            "{} staging of {} failed with: {}",
            cross!(),
            ioc.name.red().bold(),
            e
        ),
    }
}
