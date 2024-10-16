use git2::{Repository, RepositoryState};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;

/// struct for origin information
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Origin {
    directory: PathBuf,
    remote: String,
    branch: String,
    message: String,
    commit: String,
    time: String,
    tag: String,
    author: String,
    state: String,
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
        let mut origin: Origin = Default::default();
        origin.directory = dir.as_ref().canonicalize().unwrap_or_default();

        if let Ok(repo) = Repository::discover(dir) {
            // Print the branch name
            if let Ok(head) = repo.head() {
                if let Some(name) = head.shorthand() {
                    origin.branch = name.to_owned();
                    println!("Branch: {}", origin.branch);
                }
            }

            // Print the last commit details
            if let Ok(reference) = repo.head() {
                if let Some(oid) = reference.target() {
                    if let Ok(commit) = repo.find_commit(oid) {
                        origin.author = commit.author().name().unwrap_or_default().to_owned();
                        origin.message = commit.summary().unwrap_or_default().to_owned();
                        origin.commit = oid.to_string();
                        origin.time = NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0)
                            .unwrap_or_default()
                            .to_string();
                        println!("Author: {}", origin.author);
                        println!("Message: {}", origin.message);
                        println!("Commit: {}", origin.commit);
                        println!("Time: {}", origin.time);
                    }
                }
            }

            // Print the remote URL
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    origin.remote = url.to_owned();
                }
            }

            // Check if the current commit is tagged
            if let Ok(tag_name) = repo.describe(git2::DescribeOptions::new().describe_tags()) {
                origin.tag = tag_name.format(None).unwrap_or_default();
                // println!("Tagged Version: {}", tag_name.format(None).unwrap());
            }

            // Print the repository state (e.g., if it's clean, has uncommitted changes, etc.)
            match repo.state() {
                RepositoryState::Clean => origin.state = "clean".to_owned(),
                _ => origin.state = format!("{:?}", repo.state()),
            }
        }
        origin
    }

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
