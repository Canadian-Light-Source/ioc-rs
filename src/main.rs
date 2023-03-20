use std::env;


use clap::{Parser};

use tera::{Context, Tera, Error};
use users::get_current_username;
use chrono::prelude::*;

use std::path::{Path, PathBuf};

use std::fs::{File};
use std::io::prelude::*;
use std::fs;

// my mods
pub mod cli;
use cli::{Cli, Commands};

/// IOC structure
#[derive(Debug)]
struct IOC{
    /// name of the IOC
    name: String,
    /// source of the IOC definition
    source: PathBuf,
    /// staging directory
    stage: PathBuf,
    /// data directory for checksum
    data: PathBuf,
    /// deploy directory for IOC
    destination: PathBuf,
}

/// IOC structure implementation
impl IOC {
    /// Creates a new IOC structure
    fn new(
        name: &String,
        stage_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>)
        -> Result<IOC, &'static str> {
        let mut working_dir = env::current_dir().unwrap();
        if name.len()>0 {
            working_dir = working_dir.as_path().join(&name);
        }
        let stage = stage_root.as_ref().join(&name);
        let destination = destination_root.as_ref().join(&name);
        let data = destination_root.as_ref().join("data").join(&name);
        // check source exists
        match working_dir.is_dir() {
            true => Ok(IOC {
                name: name.to_string(),
                source: working_dir,
                stage: stage,
                data: data,
                destination: destination,
            }),
            false => Err("no such file or directory")
        }

    }

    fn render(&self) -> Result<String, Error> {

        let user_name = match get_current_username(){
            Some(uname) => uname,
            None => "unkown".into(),
        };

        let tera = match Tera::new("templates/*.tera") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };

        let local: DateTime<Local> = Local::now();
        let formatted = format!("{}", local.format("%Y-%m-%d %H:%M:%S.%f"));
        let mut context = Context::new();
        context.insert("IOC", &self.name);
        context.insert("user", &user_name.as_os_str().to_str());
        context.insert("destination", &self.destination);
        context.insert("date", &formatted);

        tera.render("startup.tera", &context)
    }

    fn wrap_startup(&self) -> std::io::Result<()> {
        let old = &self.stage.join("startup.iocsh");
        let ioc_startup = "startup.iocsh_".to_owned() + &self.name;
        let new = &self.stage.join(ioc_startup);
        fs::copy(old.as_path(), new.as_path())?;
        write_file(&old, self.render().unwrap())?;
        Ok(())
    }

    fn stage(&self) -> std::io::Result<()> {
        println!("staging: {:?}", self.name);
        if self.stage.exists(){
            fs::remove_dir_all(&self.stage)?;  // prep stage
        }
        copy_recursively(&self.source, &self.stage)?;
        self.wrap_startup()?;
        Ok(())
    }

    fn deploy(&self) -> std::io::Result<()> {
        println!("deploying: {:?}", self.name);
        if self.destination.exists(){
            fs::remove_dir_all(&self.destination)?;  // prep stage
        }
        copy_recursively(&self.stage, &self.destination)?;
        Ok(())
    }

    /// check whether destination has been tempered with
    fn check_destination(&self) -> bool {
        let dst = self.destination.as_path().join(&self.name);
        // 1. exists
        let exists = dst.is_dir();
        // 2. has_md5
        // 3. calc md5 from destination
        // 4. is md5 identical
        exists
    }
}


fn collect_iocs(ioc_names: &Vec<String>, stage_root: impl AsRef<Path>, destination_root: impl AsRef<Path>) -> Vec<IOC>{
    let mut iocs: Vec<IOC> = Vec::new();
    for name in ioc_names.iter() {
        match IOC::new(name, &stage_root, &destination_root) {
            Ok(new_ioc) => iocs.push(new_ioc),
            _ => ()
        };
    }
    iocs
}

fn write_file(file_name: impl AsRef<Path>, content: String) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Copy files from source to destination recursively.
pub fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            // copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            copy_recursively(entry.path(), destination.as_ref())?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // let destination = env::current_dir().unwrap().as_path().join("TEST/ioc");
    let stage_root = "stage/";
    let deploy_root = "deploy/ioc/";


    let mut ioc_list: Vec<IOC> = Vec::new();

    match &cli.command {
        Some(Commands::Install { dryrun, iocs }) => {
            println!("INSTALL");
            println!("\t dryrun: {}", dryrun);
            match iocs {
                Some(i) => ioc_list = collect_iocs(i, &stage_root, &deploy_root),
                None => panic!(),
            };
            // call install routine
            // install_iocs(targets, &destination)
        }
        None => println!("NO ACTION --> BYE")
    }

    for ioc in &ioc_list {
        println!("{:?}", ioc);
        println!("destination exists: {}", ioc.check_destination());
        _= ioc.stage();
        _= ioc.deploy();
        println!("{}",ioc.render().unwrap());
    }
}
