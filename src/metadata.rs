use log::trace;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const NAME: &str = env!("CARGO_PKG_NAME");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
pub struct PackageData<'a> {
    name: &'a str,
    description: &'a str,
    version: &'a str,
    authors: &'a str,
    repo: &'a str,
}

impl PackageData<'_> {
    pub fn new() -> Self {
        PackageData {
            name: NAME,
            description: DESCRIPTION,
            version: VERSION,
            authors: AUTHORS,
            repo: REPOSITORY,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name
    }
    pub fn get_description(&self) -> &str {
        self.description
    }
    pub fn get_version(&self) -> &str {
        self.version
    }
    pub fn get_authors(&self) -> &str {
        self.authors
    }
    pub fn get_repository(&self) -> &str {
        self.repo
    }

    pub fn report(&self) {
        trace!("  name   : {}", self.get_name());
        trace!("  desc   : {}", self.get_description());
        trace!("  version: {}", self.get_version());
        trace!("  authors: {}", self.get_authors());
        trace!("  repo:    {}", self.get_repository());
    }
}
