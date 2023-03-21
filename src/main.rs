use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// for CLI
use clap::Parser;

// for rendering templates
use chrono::prelude::*;
use tera::{Context, Error, Tera};
use users::get_current_username;

// for checksum
use blake2::{Blake2s256, Digest};
use file_hashing::get_hash_folder;

// logging
use colored::Colorize;
use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use simple_logger::SimpleLogger;

// my mods
pub mod cli;
use cli::{Cli, Commands};

macro_rules! tick {
    () => {
        "✔".green()
    };
}

macro_rules! cross {
    () => {
        "✘".red().bold()
    };
}

macro_rules! exclaim {
    () => {
        "!".yellow().bold()
    };
}

/// IOC structure
#[derive(Debug)]
struct IOC {
    /// name of the IOC
    name: String,
    /// source of the IOC definition
    source: PathBuf,
    /// staging directory
    stage: PathBuf,
    /// data directory for checksum
    data: PathBuf,
    /// hash file name
    hash_file: PathBuf,
    /// deploy directory for IOC
    destination: PathBuf,
}

/// IOC structure implementation
impl IOC {
    /// Creates a new IOC structure
    /// shold fail if source does not contain at least a 'startup.iocsh'
    // TODO: implement pre-check
    fn new(
        // name: &String,
        source: impl AsRef<Path>,
        stage_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>,
    ) -> Result<IOC, &'static str> {
        let name = source
            .as_ref()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let stage = stage_root.as_ref().join(&name);
        let destination = destination_root.as_ref().join(&name);
        let data = destination_root.as_ref().join("data").join(&name);
        let hash_file = data.join("hash");
        // check source exists
        match source.as_ref().is_dir() {
            true => Ok(IOC {
                name: name,
                source: source.as_ref().to_path_buf(),
                stage: stage,
                data: data,
                hash_file: hash_file,
                destination: destination,
            }),
            false => Err("Could not find source of IOC."),
        }
    }

    fn render(&self, template_dir: &String) -> Result<String, Error> {
        let user_name = match get_current_username() {
            Some(uname) => uname,
            None => "unkown".into(),
        };

        let tera = match Tera::new(&template_dir) {
            Ok(t) => t,
            Err(e) => {
                error!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        trace!("{} tera parser created", tick!());

        let local: DateTime<Local> = Local::now();
        let formatted = format!("{}", local.format("%Y-%m-%d %H:%M:%S.%f"));
        let mut context = Context::new();
        context.insert("IOC", &self.name);
        context.insert("user", &user_name.as_os_str().to_str());
        context.insert("destination", &self.destination);
        context.insert("date", &formatted);
        trace!(
            "{} tera context created: {:?}",
            tick!(),
            &context.clone().into_json()
        );

        trace!("{} tera rendering ...", tick!());
        tera.render("startup.tera", &context)
    }

    fn wrap_startup(&self, template_dir: &String) -> std::io::Result<()> {
        let old = &self.stage.join("startup.iocsh");
        let ioc_startup = "startup.iocsh_".to_owned() + &self.name;
        let new = &self.stage.join(ioc_startup);
        fs::copy(old.as_path(), new.as_path())?;
        trace!(
            "{} copied {:?} -> {:?}",
            tick!(),
            &old.as_path(),
            &new.as_path()
        );
        write_file(&old, self.render(template_dir).unwrap())?;
        trace!(
            "{} template rendered and written to {:?}",
            tick!(),
            &old.as_path()
        );
        Ok(())
    }

    fn stage(&self, template_dir: &String) -> std::io::Result<()> {
        trace!("staging {}", self.name.blue());
        if self.stage.exists() {
            fs::remove_dir_all(&self.stage)?; // prep stage directory
            trace!("{} {:?} removed", tick!(), &self.stage.as_path());
        }
        copy_recursively(&self.source, &self.stage)?;
        trace!(
            "{} copied {:?} -> {:?}",
            tick!(),
            &self.source.as_path(),
            &self.stage.as_path()
        );
        self.wrap_startup(template_dir)?;
        debug!("{} staging of {:?} complete.", tick!(), self.name);
        Ok(())
    }

    fn hash_ioc(&self) -> std::io::Result<()> {
        let hash = calc_directory_hash(&self.stage);
        trace!("hash: {:?}", hash);
        fs::create_dir_all(&self.data)?;
        write_file(&self.hash_file, hash)?;
        debug!(
            "{} hash_file {:?} written.",
            tick!(),
            &self.hash_file.as_path()
        );
        Ok(())
    }

    fn deploy(&self) -> std::io::Result<()> {
        trace!("deploying {}", self.name.blue());
        if self.destination.exists() {
            fs::remove_dir_all(&self.destination)?; // prep deploy directory
            trace!("removed {:?}", &self.destination);
        }
        self.hash_ioc()?;
        copy_recursively(&self.stage, &self.destination)?;
        trace!(
            "copied {:?} -> {:?}",
            &self.stage.as_path(),
            &self.destination.as_path()
        );
        debug!(
            "{} deployment of {:?} to {:?} complete.",
            tick!(),
            self.name,
            &self.destination.as_path()
        );
        Ok(())
    }

    /// check whether destination has been tempered with
    fn check_hash(&self) -> Result<String, String> {
        // destination doesn't exist yet, that's fine
        if !self.destination.exists() {
            return Ok("destination does not yet exist. No hash expected.".to_string());
        }
        let mut hash = String::from("");
        if let Ok(lines) = read_lines(&self.hash_file) {
            if let Ok(stored_hash) = lines.last().unwrap() {
                hash = stored_hash;
            };
        }

        let valid_hash = match hash == calc_directory_hash(&self.destination) {
            false => return Err("hashes do not match".to_string()),
            true => hash,
        };
        Ok(valid_hash)
    }
}

fn collect_iocs(
    ioc_names: &Vec<String>,
    stage_root: impl AsRef<Path>,
    destination_root: impl AsRef<Path>,
) -> Vec<IOC> {
    let mut iocs: Vec<IOC> = Vec::new();
    debug!("collecting iocs ...");
    for name in ioc_names.iter() {
        let work_dir = env::current_dir().unwrap().join(&name);
        trace!("working dir: {:?}", work_dir);
        match IOC::new(&work_dir, &stage_root, &destination_root) {
            Ok(new_ioc) => iocs.push(new_ioc),
            Err(e) => error!(
                "{} IOC::new failed for <{}> with: {}",
                cross!(),
                name.red().bold(),
                e
            ),
        };
    }
    iocs
}

fn write_file(file_name: impl AsRef<Path>, content: String) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Copy files from source to destination recursively.
pub fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            // copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            copy_recursively(entry.path(), destination.as_ref())?; // flatten the structure
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn calc_directory_hash(dir: impl AsRef<Path>) -> String {
    let mut hash = Blake2s256::new();
    let directory = dir.as_ref().to_str().unwrap();
    let result = get_hash_folder(&directory, &mut hash, 1, |_| {}).unwrap();
    result
}

fn main() {
    info!("IOC toolbox");
    let cli = Cli::parse();

    let stage_root = "stage/";
    let deploy_root = "deploy/ioc/";

    let l = cli.log_level.unwrap().to_lowercase();
    let log_lvl = if l == "trace" {
        LevelFilter::Trace
    } else if l == "debug" {
        LevelFilter::Debug
    } else if l == "warn" {
        LevelFilter::Warn
    } else if l == "info" {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    }; // always report errors
    SimpleLogger::new().with_level(log_lvl).init().unwrap();

    let template_dir = &cli.template_dir.unwrap_or("templates/*.tera".to_string());
    trace!("template_dir: {:?}", template_dir);

    match &cli.command {
        Some(Commands::Install {
            dryrun,
            force,
            iocs,
        }) => {
            debug!("command: <{}>", "install".yellow());
            debug!("dryrun: {}", dryrun);
            debug!("force:  {}", force);
            let ioc_list = match iocs {
                Some(i) => collect_iocs(i, &stage_root, &deploy_root),
                None => panic!(),
            };
            trace!("{} ioc list created", tick!());
            // worker
            // TODO: move to function
            for ioc in &ioc_list {
                info!("----- {} -----", ioc.name.blue().bold());
                trace!("{:?}", ioc);
                // temper check
                match ioc.check_hash() {
                    Ok(hash) => {
                        info!("{} valid hash for {} |{}|", tick!(), &ioc.name.blue(), hash);
                    }
                    Err(e) => {
                        if !force {
                            error!("{} {} --> check destination <{:?}> and use `{} {}` to deploy regardless", cross!(), e, &ioc.destination.as_path(), "ioc install --force".yellow(), &ioc.name.yellow());
                            continue;
                        } else {
                            warn!(
                                "{} failed hash check overwritten by {}",
                                exclaim!(),
                                "--force".yellow()
                            );
                        }
                    }
                }
                // staging
                trace!("staging {}", ioc.name.blue().bold());
                match ioc.stage(template_dir) {
                    Ok(_) => info!("{} staged {}", tick!(), ioc.name.blue()),
                    Err(e) => error!(
                        "{} staging of {} failed with: {}",
                        cross!(),
                        ioc.name.red().bold(),
                        e
                    ),
                }
                // deployment
                if !dryrun {
                    trace!("deploying {}", ioc.name.blue().bold());
                    match ioc.deploy() {
                        Ok(_) => info!("{} deployed {}", tick!(), ioc.name.blue()),
                        Err(e) => error!(
                            "{} deployment of {} failed with: {}",
                            cross!(),
                            ioc.name.red().bold(),
                            e
                        ),
                    }
                } else {
                    info!("{} was chosen, no deployment", "--dryrun".yellow());
                }
                trace!("------------");
            }
        }
        None => println!("NO ACTION --> BYE"),
    }
}
