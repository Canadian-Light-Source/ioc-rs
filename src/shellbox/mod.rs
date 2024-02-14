use crate::ioc;
use crate::ioc::IOC;
use log::trace;
use tera::{Context, Error, Tera};

// const SHELLBOX_CONFIG_FILE: &str = "shellbox.conf";

/// template for shellbox config
static SHELLBOX_TEMPLATE: &str =
    "{{ port }};{{ user }};{{ base_dir }};{{ command }};{{ command_args }};{{ procserv_opts }}";

#[derive(Debug, Clone, Default)]
pub struct ShellBoxConfig {
    base_dir: String,
    name: String,
    ioc_config: ioc::ioc_config::IocConfig,
}

impl ShellBoxConfig {
    /// create new ShellBoxConfig from an IOC
    pub fn from_ioc(ioc: &IOC) -> Self {
        // default to destination
        let base_dir = ioc
            .config
            .ioc
            .base_dir
            .to_owned()
            .unwrap_or(ioc.destination.to_str().unwrap_or_default().to_owned());
        ShellBoxConfig {
            base_dir,
            name: ioc.name.to_owned(),
            ioc_config: ioc.config.ioc.clone(),
        }
    }

    /// render a configuration line for shellbox
    pub fn render_shellbox_line(&self) -> Result<String, Error> {
        trace!("rendering shellbox config line for IOC: {}", self.name);
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![("shellbox_line", SHELLBOX_TEMPLATE)])
            .unwrap();
        let mut context = Context::new();
        // context.insert("IOC", &ioc.name);
        context.insert("host", &self.ioc_config.host); // default handled in struct
        context.insert("port", &self.ioc_config.port); // default handled in struct
        context.insert("user", &self.ioc_config.user); // default handled in struct
        context.insert("base_dir", &self.base_dir);
        context.insert("command", &self.ioc_config.command); // default handled in struct
        context.insert("command_args", &self.ioc_config.command_args);
        context.insert("procserv_opts", &self.ioc_config.procserv_opts);

        tera.render("shellbox_line", &context)
    }
}

#[cfg(test)]
mod tests {
    use crate::ioc::IOC;
    use crate::settings::Settings;
    use crate::shellbox;
    use std::path::Path;
    use tempfile::tempdir;

    fn get_test_ioc() -> std::io::Result<IOC> {
        let settings = Settings::build("./tests/config/test_stage.toml").unwrap();
        let template_dir = settings.get::<String>("app.template_directory").unwrap();

        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        let shellbox_root = temp_dir.path().join("shellbox");

        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            &stage_dir,
            &dest_dir,
            &shellbox_root,
            &template_dir,
        )
        .unwrap();
        Ok(test_ioc)
    }

    #[test]
    fn from_ioc_shellboxconfig() -> std::io::Result<()> {
        let test_ioc = get_test_ioc()?;

        let config = shellbox::ShellBoxConfig::from_ioc(&test_ioc);
        assert_eq!(config.ioc_config.host, "iochost");
        assert_eq!(config.ioc_config.port, 12345);
        assert_eq!(config.ioc_config.user, "control2");

        assert_eq!(config.name, "UTEST_IOC01");
        assert_eq!(config.ioc_config.command, "iocsh");
        assert_eq!(config.ioc_config.command_args, "startup.iocsh");
        assert_eq!(config.ioc_config.procserv_opts, "");

        let sbc = config.render_shellbox_line();
        assert!(sbc.is_ok());
        let exp = format!("12345;control2;{};iocsh;startup.iocsh;", config.base_dir);
        assert_eq!(sbc.unwrap(), exp);

        Ok(())
    }
}
