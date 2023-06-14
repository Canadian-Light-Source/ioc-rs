use crate::{
    ioc::IOC,
    log_macros::{cross, exclaim, tick},
};
use colored::Colorize;
use config::Config;
use log::{error, info, trace, warn};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use tera::{Context, Error, Tera};

pub fn ioc_shellbox(ioc: &IOC, settings: &Config) -> std::io::Result<()> {
    let shellbox_root = settings.get::<String>("filesystem.shellbox").unwrap();
    let shellbox_config_file = Path::new(&shellbox_root)
        .join(&ioc.config.ioc.host)
        .join("shellbox.conf");

    let cfg_line = render_shellbox_line(ioc).unwrap();

    match shellbox_config_file.exists() {
        true => {
            trace!("pre-existing config file");
            let mut hm = read_cfg(&shellbox_config_file);

            let kv = get_kv_pair(cfg_line.clone()).unwrap();
            let port = kv.0;
            let payload = kv.1;
            if is_duplicate(hm.clone(), &port, &payload) {
                error!(
                    "{} shellbox config was {} updated. Please update {:?} manually",
                    cross!(),
                    "not".red(),
                    shellbox_config_file
                );
                return Ok(());
            }

            update_hm(&mut hm, port, payload);

            let lines = hashmap_to_cfg(hm);

            let mut file = File::create(&shellbox_config_file)?;
            if let Some(string) = lines {
                file.write_all(string.as_bytes())?
            }
        }
        false => {
            warn!("create shellbox config");
            let root = Path::new(&shellbox_root).join(&ioc.config.ioc.host);
            fs::create_dir_all(root)?;
            let mut file = File::create(shellbox_config_file)?;
            file.write_all(cfg_line.as_bytes())?;
        }
    };
    info!("{} shellbox config updated.", tick!());

    Ok(())
}

// template for comma separated shellbox config
static SHELLBOX_TEMPLATE: &str =
    "{{ port }},{{ user }},{{ base_dir }},{{ command }},{{ procserv_opts }}";

fn render_shellbox_line(ioc: &IOC) -> Result<String, Error> {
    let conf = ioc.clone().config;

    let base_dir = match conf.ioc.base_dir {
        Some(ioc_base_dir) => {
            warn!("{} non-default work_dir: {}", exclaim!(), ioc_base_dir);
            ioc_base_dir
        }
        None => ioc.destination.to_str().unwrap().to_string(),
    };

    let command = match conf.ioc.command {
        Some(opts) => {
            trace!("command: {}", opts);
            opts
        }
        None => format!("iocsh -n {}", ioc.name),
    };

    let procserv_opts = match conf.ioc.procserv_opts {
        Some(opts) => {
            trace!("procServ opts: {}", opts);
            opts
        }
        None => "".to_string(),
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("sb_line", SHELLBOX_TEMPLATE)])
        .unwrap();
    let mut context = Context::new();
    // context.insert("IOC", &ioc.name);
    context.insert("host", &conf.ioc.host);
    context.insert("port", &conf.ioc.port);
    context.insert("user", &conf.ioc.user); // default handled in struct
    context.insert("base_dir", &base_dir);
    context.insert("command", &command);
    context.insert("procserv_opts", &procserv_opts);

    tera.render("sb_line", &context)
}

/// read shellbox configuration into a hashmap with the port(s) as key(s)
fn read_cfg<P: AsRef<Path>>(filename: P) -> HashMap<String, Vec<String>> {
    let mut hashmap: HashMap<String, Vec<String>> = HashMap::new();
    let mut comments: Vec<String> =
        vec!["#- comments below this line. Lines starting with '#-' will be dropped".to_string()];

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("#-") {
            // drop lines with '#-'
            continue;
        }
        if line.starts_with('#') {
            // collect comment line
            comments.push(line);
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();

        let key = fields[0].to_owned();
        let payload = fields[1..].iter().map(|&x| x.to_owned()).collect();

        hashmap.insert(key, payload);
    }
    hashmap.insert("comments".to_string(), comments);
    hashmap
}

/// get key value pair from shellbox configuration line
fn get_kv_pair(line: String) -> Option<(String, Vec<String>)> {
    warn!("---> {}", line);
    if line.starts_with('#') {
        return None;
    }
    let fields: Vec<&str> = line.split(',').collect();

    let key = fields[0].to_owned();
    let payload = fields[1..].iter().map(|&x| x.to_owned()).collect();
    Some((key, payload))
}

/// update the hashmap, modify existing entry, or add new
fn update_hm(hashmap: &mut HashMap<String, Vec<String>>, key: String, payload: Vec<String>) {
    if let Some(existing_payload) = hashmap.get_mut(&key) {
        if payload == existing_payload.clone() {
            return; // there is nothing to do here
        }
        trace!("existing value for {} -> {:?}", key, payload);
        trace!("existing value for {} -> {:?}", key, existing_payload);
        *existing_payload = existing_payload.clone(); // force clone to update the value
        existing_payload.clear(); // clear the existing values
        existing_payload.extend_from_slice(&payload); // insert new values
    } else {
        trace!("new entry for {} -> {:?}", key, payload);
        hashmap.insert(key, payload);
    }
}

/// transform hashmap into multiline string for writing to file
fn hashmap_to_cfg(hashmap: HashMap<String, Vec<String>>) -> Option<String> {
    if hashmap.is_empty() {
        error!("{} empty hashmap. This should _not_ happen!", cross!());
        return None;
    }
    let mut hashmap = hashmap.clone(); // mutable clone
    let comments = hashmap.remove("comments").unwrap_or(vec!["#-".to_string()]);
    let mut result = String::new();

    for (key, value) in hashmap {
        let mut s = key + ",";
        s += &value.join(",");
        s += "\n";
        result += &s;
    }
    for line in comments {
        let mut s = line;
        s += "\n";
        result += &s;
    }
    Some(result)
}

fn is_duplicate(hashmap: HashMap<String, Vec<String>>, port: &str, payload: &[String]) -> bool {
    for (key, value) in hashmap {
        if (value == payload) && (key != port) {
            error!(
                "{} new config for port {} has duplicate payload for existing port {}",
                cross!(),
                key.red(),
                port.yellow()
            );
            return true;
        }
    }
    false
}
