use std::ffi::OsString;
use std::path::Path;
use std::{
    fs::{self, File},
    io::Write,
};

use chrono::{DateTime, Local};
use colored::Colorize;
use log::{error, trace};
use tera::{Context, Error, Tera};
// use users::get_current_username;

use crate::{ioc::IOC, log_macros::tick, metadata::PackageData};

fn base_context() -> Context {
    let metadata = PackageData::new();
    let local: DateTime<Local> = Local::now();
    let formatted = format!("{}", local.format("%Y-%m-%d %H:%M:%S.%f"));
    let mut context = Context::new();
    context.insert("tool", metadata.get_name());
    context.insert("version", metadata.get_version());
    context.insert("date", &formatted);
    context
}

fn create_parser(template_dir: &str) -> Tera {
    let tera = match Tera::new(template_dir) {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    trace!("{} tera parser created", tick!());
    tera
}

fn get_user_name() -> Option<OsString> {
    match users::get_current_username() {
        Some(uname) => Some(uname),
        None => std::env::var_os("USER").or(Some("unkown".into())),
    }
}

fn render_startup_script(ioc: &IOC, template_dir: &str) -> Result<String, Error> {
    let user_name = get_user_name().expect("failed to get username");

    let tera = create_parser(template_dir);

    let mut context = base_context();
    context.insert("IOC", &ioc.name);
    context.insert("user", &user_name.as_os_str().to_str());
    context.insert("destination", &ioc.destination);
    // context.insert("date", &formatted);
    trace!(
        "{} tera context created: {:?}",
        tick!(),
        &context.clone().into_json()
    );

    trace!("{} tera rendering ...", tick!());
    tera.render("startup.tera", &context)
}

pub fn render_startup(ioc: &IOC, template_dir: &str) -> std::io::Result<()> {
    if !Path::new(template_dir).is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The specified template path is not a directory.",
        ));
    }
    let old = &ioc.stage.join("startup.iocsh");
    let ioc_startup = "startup.iocsh_".to_owned() + &ioc.name;
    let new = &ioc.stage.join(ioc_startup);
    fs::copy(old.as_path(), new.as_path())?;
    trace!(
        "{} copied {:?} -> {:?}",
        tick!(),
        &old.as_path(),
        &new.as_path()
    );
    let mut file = File::create(old)?;
    file.write_all(render_startup_script(ioc, template_dir).unwrap().as_bytes())?;
    trace!(
        "{} template rendered and written to {:?}",
        tick!(),
        &old.as_path()
    );
    Ok(())
}

#[cfg(test)]
mod render_tests {
    use crate::test_utils::new_test_ioc;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn startup() {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();
        let expected = "\
# ------------------
# TEST HEADER
# ------------------

< startup.iocsh_UTEST_IOC01

# ------------------
# TEST FOOTER
# ------------------

";

        let template_dir = "./tests/render_test/templates/*.tera";
        assert_eq!(
            render_startup_script(&test_ioc, template_dir).unwrap(),
            expected
        );
    }
}
