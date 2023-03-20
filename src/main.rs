use std::path::{Path, PathBuf};
use std::fs::{File};
use std::io::prelude::*;
use std::fs;
use std::io::{self, BufRead};
use std::env;

// for CLI
use clap::{Parser};

// for rendering templates
use tera::{Context, Tera, Error};
use users::get_current_username;
use chrono::prelude::*;

// for checksum
use blake2::{Blake2s256, Digest};
use file_hashing::get_hash_folder;

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
    /// hash file name
    hash_file: PathBuf,
    /// deploy directory for IOC
    destination: PathBuf,
}

/// IOC structure implementation
impl IOC {
    /// Creates a new IOC structure
    fn new(
        name: &String,
        source: impl AsRef<Path>,
        stage_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>)
        -> Result<IOC, &'static str> {
        let stage = stage_root.as_ref().join(&name);
        let destination = destination_root.as_ref().join(&name);
        let data = destination_root.as_ref().join("data").join(&name);
        let hash_file = data.join("hash");
        // check source exists
        match source.as_ref().is_dir() {
            true => Ok(IOC {
                name: name.to_string().replace("/", ""),
                source: source.as_ref().to_path_buf(),
                stage: stage,
                data: data,
                hash_file: hash_file,
                destination: destination,
            }),
            false => Err("Could not find source of IOC. skipping.")
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

    // fn pre_check(&self) {
    //     if self.destination.exists(){
    //         let hash = self.check_hash();
    //         match hash {
    //             Ok(h) => (),
    //             Err(e) => {
    //                 println!("The destination {:?} was tempered with.\nError: {}", &self.destination, e);
    //             }
    //         }
    //     }
    // }

    fn stage(&self) -> std::io::Result<()> {
        println!("staging: {:?}", self.name);
        if self.stage.exists(){
            fs::remove_dir_all(&self.stage)?;  // prep stage directory
        }
        copy_recursively(&self.source, &self.stage)?;
        self.wrap_startup()?;
        Ok(())
    }

    fn hash_ioc(&self) -> std::io::Result<()> {
        let hash = calc_directory_hash(&self.stage);
        fs::create_dir_all(&self.data)?;
        write_file(&self.hash_file, hash)?;
        Ok(())
    }

    fn deploy(&self) -> std::io::Result<()> {
        println!("deploying: {:?}", self.name);
        if self.destination.exists(){
            fs::remove_dir_all(&self.destination)?;  // prep deploy directory
        }
        self.hash_ioc()?;
        copy_recursively(&self.stage, &self.destination)?;
        Ok(())
    }

    /// check whether destination has been tempered with
    fn check_hash(&self) -> Result<String, String> {
        let mut hash = String::from("");
        if let Ok(lines) = read_lines(&self.hash_file) {
            if let Ok(stored_hash) = lines.last().unwrap(){
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


fn collect_iocs(ioc_names: &Vec<String>, stage_root: impl AsRef<Path>, destination_root: impl AsRef<Path>) -> Vec<IOC>{
    let mut iocs: Vec<IOC> = Vec::new();
    println!("collecting iocs ...");
    // if ioc_names.len() == 1 {
    //     let pwd = env::current_dir().unwrap();
    //     let name = pwd.file_stem().unwrap().to_str().unwrap();
    //     println!("just the one IOC: {}", &name.to_string());
    //     let new_ioc = IOC::new(&name.to_string(), &pwd, &stage_root, &destination_root).unwrap();
    //     println!("{:?}",new_ioc);
    //     iocs.push(new_ioc);
    //     return iocs
    // }
    for name in ioc_names.iter() {
        let work_dir = env::current_dir().unwrap().join(&name);
        match IOC::new(name, &work_dir, &stage_root, &destination_root) {
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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

fn calc_directory_hash(dir: impl AsRef<Path>) -> String {
    let mut hash = Blake2s256::new();
    let directory = dir.as_ref().to_str().unwrap();
    let result = get_hash_folder(
        &directory,
        &mut hash,
        1,
        |_| {},
    )
    .unwrap();
    result
}

fn main() {
    let cli = Cli::parse();

    // let destination = env::current_dir().unwrap().as_path().join("TEST/ioc");
    let stage_root = "stage/";
    let deploy_root = "deploy/ioc/";

    let mut ioc_list: Vec<IOC> = Vec::new();

    match &cli.command {
        Some(Commands::Install { dryrun, force, iocs }) => {
            println!("INSTALL");
            println!("\t dryrun: {}", dryrun);
            println!("\t force:  {}", force);
            match iocs {
                Some(i) => ioc_list = collect_iocs(i, &stage_root, &deploy_root),
                None => panic!(),
            };
            // worker
            // TODO: move to function
            for ioc in &ioc_list {
                println!("-------------------------------------------------");
                println!("{:?}", ioc);
                let hash = ioc.check_hash();
                if let Ok(ioc_hash) = &hash {
                    println!("IOC {} has valid hash {} ... proceeding", &ioc.name, ioc_hash);
                }
                if let Err(err) = &hash {
                    if !force {
                        println!("invalid hash: {}\n --> skipping <{}>", err, &ioc.name);
                        continue;
                    }
                }
                // _= ioc.pre_check();
                _= ioc.stage();
                _= ioc.deploy();
                println!("{}",ioc.render().unwrap());
            }
            // call deploy routine if not dryrun
            // install_iocs(targets, &destination)
        }
        None => println!("NO ACTION --> BYE")
    }
}
