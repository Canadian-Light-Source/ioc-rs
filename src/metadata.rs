const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const NAME: &str = env!("CARGO_PKG_NAME");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
pub struct Metadata<'a> {
    name: &'a str,
    description: &'a str,
    version: &'a str,
    authors: &'a str,
    repo: &'a str,
}

impl Metadata<'_> {
    pub fn new() -> Self {
        Metadata {
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
}
