// use crate::{
//     ioc::IOC,
//     log_macros::{cross, exclaim, tick},
// };
// use colored::Colorize;
// use log::{error, info, trace, warn};
use crate::ioc::IOC;
// use std::collections::HashMap;
// use std::fs;
// use std::fs::File;
// use std::io::{BufRead, BufReader, ErrorKind, Write};
// use std::path::Path;
// use tera::{Context, Error, Tera};

const SHELLBOX_CONFIG_FILE: &str = "shellbox.conf";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ShellBoxConfig {
    host: String,
    port: u32,
    user: String,
    name: String,
    command: String,
    procserv_opts: String,
}

impl ShellBoxConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_ioc(ioc: &IOC) -> Self {
        ShellBoxConfig {
            host: ioc.config.ioc.host.to_owned(),
            port: ioc.config.ioc.port,
            user: ioc.config.ioc.user.to_owned().unwrap_or_default(),
            name: ioc.name.to_owned(),
            command: ioc.config.ioc.command.to_owned().unwrap_or_default(),
            procserv_opts: ioc.config.ioc.procserv_opts.to_owned().unwrap_or_default(),
        }
    }
}
// #[derive(Debug)]
// pub struct Shellbox {
//     configs: HashMap<u16, HashMap<&'static str, Vec<&'static str>>>,
// }
// impl Shellbox {
//     pub fn new<P: AsRef<Path>>(dir: P) -> std::io::Result<Self> {
//         Ok(Shellbox {
//             configs: Self::from_shellbox_root(dir)?,
//         })
//     }
//     fn from_shellbox_root<P: AsRef<Path>>(
//         path: P,
//     ) -> std::io::Result<HashMap<String, HashMap<String, Vec<String>>>> {
//         let entries = fs::read_dir(path)?;
//         let mut hashmap: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
//
//         for entry in entries {
//             let entry = entry?;
//             if entry.file_type()?.is_dir() {
//                 let hostname = entry.file_name().to_str().unwrap().to_string();
//                 hashmap.insert(hostname, read_cfg(entry.path().join(SHELLBOX_CONFIG_FILE)));
//             }
//         }
//         Ok(hashmap)
//     }
//     pub fn get_config(self, hostname: &str) -> std::io::Result<HashMap<String, Vec<String>>> {
//         Ok(self.configs.get(hostname).unwrap().to_owned())
//     }
// }
//
// pub fn ioc_shellbox(ioc: &IOC) -> std::io::Result<()> {
//     let shellbox_root = &ioc.shellbox_root.join(&ioc.config.ioc.host);
//     let shellbox_config_file = Path::new(&shellbox_root).join(SHELLBOX_CONFIG_FILE);
//
//     let cfg_line = render_shellbox_line(ioc).unwrap();
//
//     let mut hm = read_cfg(&shellbox_config_file);
//     let (port, payload) = get_kv_pair(&cfg_line.as_str().clone());
//     if is_duplicate(&hm, &port, &payload) {
//         error!(
//             "{} {}: identical IOC entry detected on a different port!",
//             cross!(),
//             ioc.name.red(),
//         );
//         error!(
//             "{} shellbox config was {} updated. Please update {:?} manually",
//             cross!(),
//             "not".red(),
//             shellbox_config_file
//         );
//         return Err(std::io::Error::new(
//             ErrorKind::AlreadyExists,
//             "duplicate in shellbox.conf",
//         ));
//     }
//     update_hm(&mut hm, port.to_owned(), payload);
//
//     let content = match hashmap_to_cfg(hm) {
//         Some(lines) => lines,
//         None => cfg_line,
//     };
//
//     // write to file
//     let root = Path::new(&shellbox_root);
//     fs::create_dir_all(root)?;
//     let mut file = File::create(&shellbox_config_file)?;
//     file.write_all(content.as_bytes())?;
//
//     info!("{} shellbox config updated.", tick!());
//
//     Ok(())
// }
//
// // template for comma separated shellbox config
// static SHELLBOX_TEMPLATE: &str =
//     "{{ port }},{{ user }},{{ base_dir }},{{ command }},{{ procserv_opts }}";
// fn render_shellbox_line(ioc: &IOC) -> Result<String, Error> {
//     let conf = ioc.clone().config;
//
//     let base_dir = match conf.ioc.base_dir {
//         Some(ioc_base_dir) => {
//             warn!("{} non-default work_dir: {}", exclaim!(), ioc_base_dir);
//             ioc_base_dir
//         }
//         None => ioc.destination.to_str().unwrap().to_string(),
//     };
//
//     let command = match conf.ioc.command {
//         Some(opts) => {
//             trace!("command: {}", opts);
//             opts
//         }
//         None => format!("iocsh -n {} startup.iocsh", ioc.name),
//     };
//
//     let procserv_opts = match conf.ioc.procserv_opts {
//         Some(opts) => {
//             trace!("procServ opts: {}", opts);
//             opts
//         }
//         None => "".to_string(),
//     };
//
//     let mut tera = Tera::default();
//     tera.add_raw_templates(vec![("sb_line", SHELLBOX_TEMPLATE)])
//         .unwrap();
//     let mut context = Context::new();
//     // context.insert("IOC", &ioc.name);
//     context.insert("host", &conf.ioc.host);
//     context.insert("port", &conf.ioc.port);
//     context.insert("user", &conf.ioc.user); // default handled in struct
//     context.insert("base_dir", &base_dir);
//     context.insert("command", &command);
//     context.insert("procserv_opts", &procserv_opts);
//
//     tera.render("sb_line", &context)
// }

// /// read shellbox configuration into a hashmap with the port(s) as key(s)
// fn read_cfg<P: AsRef<Path>>(filename: P) -> HashMap<String, Vec<String>> {
//     let mut hashmap: HashMap<String, Vec<String>> = HashMap::new();
//     let comments: Vec<String> =
//         vec!["#- comments below this line. Lines starting with '#-' will be dropped".to_string()];
//
//     match File::open(filename) {
//         Ok(f) => cfg_hashmap(f, hashmap, comments),
//         Err(_) => {
//             hashmap.insert("comments".to_string(), comments);
//             hashmap
//         }
//     }
// }
// fn cfg_hashmap(
//     file: File,
//     mut hashmap: HashMap<String, Vec<String>>,
//     mut comments: Vec<String>,
// ) -> HashMap<String, Vec<String>> {
//     BufReader::new(file)
//         .lines()
//         .filter_map(Result::ok)
//         .filter(|line| !line.starts_with("#-"))
//         .map(|line| {
//             if line.starts_with('#') {
//                 Some((true, (line, vec![""]))) // Collect comment lines
//             } else {
//                 Some((false, get_kv_pair(line.as_str()))) // Store key-value pairs in a tuple
//             }
//         })
//         .for_each(|item| {
//             if let Some((is_comment, data)) = item {
//                 let (key, payload) = data;
//                 if is_comment {
//                     comments.push(key);
//                 } else {
//                     hashmap.insert(key, payload);
//                 }
//             }
//         });
//     hashmap.insert("comments".to_string(), comments);
//     hashmap
// }

// get key value pair from shellbox configuration line
// fn get_kv_pair(line: &str) -> (u16, Vec<&str>) {
//     let fields: Vec<&str> = line.split(',').collect();
//
//     let key = fields[0].trim_start().parse::<u16>().unwrap_or_default();
//     let payload = fields[1..].iter().map(|&x| x.trim()).collect();
//     (key, payload)
// }
//
// /// update the hashmap, modify existing entry, or add new
// fn update_hm(hashmap: &mut HashMap<String, Vec<String>>, key: String, payload: Vec<String>) {
//     if let Some(existing_payload) = hashmap.get_mut(&key) {
//         if payload == existing_payload.clone() {
//             return; // there is nothing to do here
//         }
//         trace!("existing value for {} -> {:?}", key, payload);
//         trace!("existing value for {} -> {:?}", key, existing_payload);
//         *existing_payload = existing_payload.clone(); // force clone to update the value
//         existing_payload.clear(); // clear the existing values
//         existing_payload.extend_from_slice(&payload); // insert new values
//     } else {
//         trace!("new entry for {} -> {:?}", key, payload);
//         hashmap.insert(key, payload);
//     }
// }
//
// /// transform hashmap into multiline string for writing to file
// fn hashmap_to_cfg(hashmap: HashMap<String, Vec<String>>) -> Option<String> {
//     if hashmap.is_empty() {
//         error!("{} empty hashmap. This should _not_ happen!", cross!());
//         return None;
//     }
//     let mut hashmap = hashmap.clone(); // mutable clone
//     let comments = hashmap.remove("comments").unwrap_or(vec!["#-".to_string()]);
//     let mut result = String::new();
//
//     for (key, value) in hashmap {
//         let mut s = key + ",";
//         s += &value.join(",");
//         s += "\n";
//         result += &s;
//     }
//     for line in comments {
//         let mut s = line;
//         s += "\n";
//         result += &s;
//     }
//     Some(result)
// }
//
// fn is_duplicate(hashmap: &HashMap<String, Vec<String>>, port: &str, payload: &[String]) -> bool {
//     hashmap
//         .iter()
//         .any(|(key, value)| (value == payload) && (key != port))
// }
//
#[cfg(test)]
mod tests {
    use crate::ioc::IOC;
    use crate::settings::Settings;
    use crate::shellbox;
    // use std::io;
    use std::path::Path;
    use tempfile::tempdir;
    // use std::io;
    // use tempfile::tempdir;

    // #[test]
    // fn create_shellbox() -> io::Result<()> {
    //     let temp_dir = tempdir()?;
    //     assert!(shellbox::Shellbox::new(temp_dir).is_ok());
    //     Ok(())
    // }

    // #[test]
    // fn get_kv_pair() {
    //     let input = " 12345,kiveln, /ioc/MTEST-counter02,    iocsh -7.0.7, -n MTEST-counter02 startup.script, -w -l 50001 foo bar 1@%$/*()";
    //     let (k, v) = shellbox::get_kv_pair(input);
    //     assert_eq!(k, 12345);
    //     assert_eq!(v[0], "kiveln");
    //     assert_eq!(v[1], "/ioc/MTEST-counter02");
    //     assert_eq!(v[2], "iocsh -7.0.7");
    // }
    #[test]
    fn new_shellboxconfig() {
        let config = shellbox::ShellBoxConfig::new();
        assert_eq!(config.host, "");
        assert_eq!(config.port, 0u16);
        assert_eq!(config.user, "");
        assert_eq!(config.name, "");
        assert_eq!(config.command, "");
        assert_eq!(config.procserv_opts, "");
    }

    #[test]
    fn mut_shellboxconfig() {
        let mut config = shellbox::ShellBoxConfig::new();

        config.host = "localhost".to_owned();
        config.port = 10001u16;
        config.user = "nobody".to_owned();
        config.name = "MTEST-10001".to_owned();
        config.command = "iocsh".to_owned();
        config.procserv_opts = "--allow -w".to_owned();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 10001);
        assert_eq!(config.user, "nobody");
        assert_eq!(config.name, "MTEST-10001");
        assert_eq!(config.command, "iocsh");
        assert_eq!(config.procserv_opts, "--allow -w");
    }

    #[test]
    fn from_ioc_shellboxconfig() -> std::io::Result<()> {
        let settings = Settings::build("./tests/config/test_stage.toml").unwrap();
        let template_dir = settings.get::<String>("app.template_directory").unwrap();

        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        let shellbox_root = temp_dir.path().join("shellbox");

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            stage_dir,
            dest_dir,
            shellbox_root,
            template_dir,
        )
        .unwrap();

        let config = shellbox::ShellBoxConfig::from_ioc(&test_ioc);
        assert_eq!(config.host, "iochost");
        assert_eq!(config.port, 12345);
        assert_eq!(config.user, "control2");
        assert_eq!(config.name, "UTEST_IOC01");
        assert_eq!(config.command, "iocsh");
        assert_eq!(config.procserv_opts, "");
        Ok(())
    }
}
