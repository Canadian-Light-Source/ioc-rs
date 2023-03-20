use std::path::PathBuf;
use std::env;

use clap::{Parser};
pub mod cli;
use cli::{Cli, Commands};

/// IOC structure
#[derive(Debug)]
struct IOC{
    name: String,
    source: PathBuf,
    destination: PathBuf,
}

/// IOC structure implementation
impl IOC {
    /// Creates a new IOC structure
    fn new(name: &String, destination: &PathBuf) -> Result<IOC, &'static str> {
        let mut working_dir = env::current_dir().unwrap();
        if name.len()>0{
            working_dir = working_dir.as_path().join(&name);
        }
        // check source exists
        match working_dir.is_dir() {
            true => Ok(IOC { name: name.to_string(), source: working_dir, destination: destination.to_path_buf() }),
            false => Err("no such file or directory")
        }
        
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

// fn install_iocs(iocs: &Vec<String>, dst: &PathBuf){
//     for ioc in iocs{
//         install_ioc(ioc, dst);
//     }
// }



fn collect_iocs(ioc_names: &Vec<String>, destination: &PathBuf) -> Vec<IOC>{
    let mut IOCs: Vec<IOC> = Vec::new();
    for name in ioc_names.iter() {
        match IOC::new(name, &destination) {
            Ok(new_ioc) => IOCs.push(new_ioc),
            _ => ()
        };
    }
    IOCs
}

fn main() {
    let cli = Cli::parse();

    let destination = env::current_dir().unwrap().as_path().join("TEST/ioc");

    let mut IOCs: Vec<IOC> = Vec::new();

    match &cli.command {
        Some(Commands::Install { dryrun, iocs }) => {
            println!("INSTALL");
            println!("\t dryrun: {}", dryrun);
            match iocs {
                Some(i) => IOCs = collect_iocs(i, &destination),
                None => panic!(),
            };
            // call install routine
            // install_iocs(targets, &destination)
        }
        None => println!("NO ACTION --> BYE")
    }

    for ioc in &IOCs {
        println!("{:?}", ioc);
        println!("destination exists: {}", ioc.check_destination());
    }
}
