use std::path::Path;
use std::{env, fs, io};

use colored::Colorize;
use config::Config;
use log::{debug, error, info, trace};

use crate::{
    ioc::IOC,
    log_macros::{cross, tick},
    render,
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
    match stage(&ioc, template_dir.as_str()) {
        Ok(_) => info!("{} staged {}", tick!(), ioc.name.blue()),
        Err(e) => error!(
            "{} staging of {} failed with: {}",
            cross!(),
            ioc.name.red().bold(),
            e
        ),
    }
}

pub fn stage(ioc: &IOC, template_dir: &str) -> std::io::Result<()> {
    trace!("staging {}", ioc.name.blue());
    if ioc.stage.exists() {
        remove_dir_contents(&ioc.stage)?; // prep stage directory
        trace!("{} {:?} removed", tick!(), &ioc.stage.as_path());
    }
    copy_recursively(&ioc.source, &ioc.stage)?;
    trace!(
        "{} copied {:?} -> {:?}",
        tick!(),
        &ioc.source.as_path(),
        &ioc.stage.as_path()
    );
    render::render_startup(ioc, template_dir)?;
    debug!("{} staging of {:?} complete.", tick!(), ioc.name);
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
