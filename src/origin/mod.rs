use git2::Repository;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// struct for origin information
/// directory: mandatory source directory
/// url: optional for git repositories
/// branch_name: optional for git repositories
/// tag: optional for git repositories
#[derive(Debug, Serialize, Deserialize)]
pub struct Origin {
    directory: PathBuf,
    url: String,
    branch_nane: String,
    message: String,
    tag: String,
}

impl Origin {
    /// new Origin
    /// tries if `dir` is a git repo, if not the Origin is populated with the path.
    ///
    /// # Arguments
    ///
    /// * `dir` - fs directory.
    ///
    /// # Returns
    ///
    /// Origin
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        match Self::from_git_dir(&dir).or(Self::from_dir(&dir)) {
            Ok(o) => o,
            Err(e) => panic!("{}", e),
        }
    }

    /// Origin from directory
    /// This is a fallback.
    /// # Arguments
    ///
    /// * `dir` - plain fs directory.
    ///
    /// # Returns
    ///
    /// Origin
    fn from_dir<P>(dir: P) -> std::io::Result<Origin>
    where
        P: AsRef<Path>,
    {
        match dir.as_ref().is_dir() {
            true => Ok(Origin {
                directory: dir.as_ref().canonicalize().unwrap(),
                url: "no url".to_string(),
                branch_nane: "".to_string(),
                message: "".to_string(),
                tag: "".to_string(),
            }),
            false => Err(std::io::Error::new(
                // std::io::ErrorKind::NotADirectory,   // only in nightly as of 2023-06-30 // TODO: change as soon as it's in stable
                std::io::ErrorKind::Other,
                "not a directory",
            )),
        }
    }

    /// Origin from git directory
    /// # Arguments
    ///
    /// * `git_dir` - directory of the git repo.
    ///
    /// # Returns
    ///
    /// Origin
    fn from_git_dir<P>(git_dir: P) -> Result<Origin, git2::Error>
    where
        P: AsRef<Path>,
    {
        // Open the repository
        let repo = Repository::open(&git_dir)?;

        // Get the active branch
        let branch = repo.head()?;
        let branch_name = branch.shorthand().unwrap().to_string();

        // Get the latest commit message
        let latest_commit = branch.peel_to_commit()?;
        let commit_message = latest_commit
            .message()
            .unwrap_or("no commit message")
            .to_string();

        // Get the remote url
        let remote = repo.find_remote("origin")?;
        let remote_url = remote.url().unwrap_or("no remote url").to_string();

        Ok(Origin {
            directory: repo.path().to_owned(),
            url: remote_url,
            branch_nane: branch_name,
            message: commit_message,
            tag: "no tag".to_string(),
        })
    }

    /// write file `path/ORIGIN`
    /// # Arguments
    ///
    /// * `path` - path to write the file to.
    ///
    /// # Returns
    ///
    /// std::io:Result()
    ///
    /// # Examples
    /// ```
    /// fn main() -> io::Result<()> {
    ///     let git_origin = Origin::new("./");
    ///     git_origin.write_origin_file("./")?;
    /// }
    /// ```
    pub fn write_origin_file<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        // Serialize the struct to a YAML string
        let yaml_string = serde_yaml::to_string(&self).unwrap();

        // Write the YAML string to a file
        let mut file = File::create(path.as_ref().join("ORIGIN"))?;
        file.write_all(yaml_string.as_bytes())?;

        Ok(())
    }
}
