use crate::log_macros::exclaim;
use colored::Colorize;
use log::warn;
use std::path::Path;
use std::{fs, io};

#[derive(Debug, Clone)]
pub enum CopyMode {
    /// Copy directory structure as-is
    Preserve,
    /// Flatten all files to destination root, except for preserved directories
    FlattenExcept(Vec<String>),
}

impl CopyMode {
    pub fn preserve_directories() -> Self {
        Self::FlattenExcept(vec!["cfg".to_string()])
    }
}

fn should_skip_entry(entry_name: &str, destination_parent: &Path) -> bool {
    // Skip if destination is in the source directory (prevent recursion)
    if entry_name.contains(&destination_parent.to_string_lossy().as_ref()) {
        warn!(
            "{} skipping recursion: {} --> {:?}",
            exclaim!(),
            entry_name,
            destination_parent
        );
        return true;
    }

    // Skip hidden files/directories and specific unwanted directories
    let skip_patterns = [".", "__pycache__"];
    if skip_patterns
        .iter()
        .any(|pattern| entry_name.starts_with(pattern))
    {
        return true;
    }

    false
}

fn get_entry_name(entry: &fs::DirEntry) -> io::Result<String> {
    entry
        .file_name()
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in filename"))
}

fn copy_directory(
    entry_path: &Path,
    entry_name: &str,
    destination: &Path,
    copy_mode: &CopyMode,
) -> io::Result<()> {
    match copy_mode {
        CopyMode::Preserve => {
            copy_recursively(entry_path, destination.join(entry_name), copy_mode.clone())
        }
        CopyMode::FlattenExcept(preserved_dirs) => {
            if preserved_dirs.contains(&entry_name.to_string()) {
                // Keep directory structure for preserved directories
                copy_recursively(entry_path, destination.join(entry_name), copy_mode.clone())
            } else {
                // Flatten: copy contents directly to destination
                copy_recursively(entry_path, destination, copy_mode.clone())
            }
        }
    }
}

fn copy_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::copy(source, destination)?;
    Ok(())
}

pub fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            remove_dir_contents(&path)?;
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Copy files from source to destination recursively.
pub fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    copy_mode: CopyMode,
) -> io::Result<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_name = get_entry_name(&entry)?;

        // Skip unwanted entries
        if should_skip_entry(&entry_name, destination.parent().unwrap_or(destination)) {
            continue;
        }

        let entry_path = entry.path();

        if entry.file_type()?.is_dir() {
            copy_directory(&entry_path, &entry_name, destination, &copy_mode)?;
        } else {
            copy_file(&entry_path, &destination.join(&entry_name))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn copy_files() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(source_dir.join("cfg"))?;
        fs::create_dir_all(source_dir.join("nested_dir"))?;
        fs::write(source_dir.join("empty.txt"), "file in root")?;
        fs::write(
            source_dir.join("nested_dir/nested_file.txt"),
            "file in nested_dir -> ends up in root",
        )?;
        fs::write(source_dir.join("cfg/file_in_cfg_dir.txt"), "file in cfg")?;

        let target_dir = temp_dir.path().join("target");

        let empty_file = target_dir.join("empty.txt");
        let nested_file = target_dir.join("nested_file.txt");
        let cfg_file = target_dir.join("cfg/file_in_cfg_dir.txt");

        copy_recursively(source_dir, &target_dir, CopyMode::preserve_directories()).unwrap();

        // check if all files made it
        assert!(empty_file.exists());
        assert!(cfg_file.exists());
        assert!(nested_file.exists());
        Ok(())
    }

    #[test]
    fn copy_files_preserve_structure() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(source_dir.join("nested_dir"))?;
        fs::write(source_dir.join("empty.txt"), "file in root")?;
        fs::write(
            source_dir.join("nested_dir/nested_file.txt"),
            "file in nested_dir",
        )?;

        let target_dir = temp_dir.path().join("target");

        copy_recursively(source_dir, &target_dir, CopyMode::Preserve).unwrap();

        // check structure is preserved
        assert!(target_dir.join("empty.txt").exists());
        assert!(target_dir.join("nested_dir/nested_file.txt").exists());
        Ok(())
    }

    #[test]
    fn copy_files_custom_preserve_dirs() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(source_dir.join("templates"))?;
        fs::create_dir_all(source_dir.join("docs"))?;
        fs::create_dir_all(source_dir.join("flatten_me"))?;
        fs::write(
            source_dir.join("templates/template.txt"),
            "template content",
        )?;
        fs::write(source_dir.join("docs/readme.txt"), "docs content")?;
        fs::write(
            source_dir.join("flatten_me/should_be_flat.txt"),
            "flattened content",
        )?;

        let target_dir = temp_dir.path().join("target");

        copy_recursively(
            source_dir,
            &target_dir,
            CopyMode::FlattenExcept(vec!["templates".to_string(), "docs".to_string()]),
        )
        .unwrap();

        // check preserved directories maintain structure
        assert!(target_dir.join("templates/template.txt").exists());
        assert!(target_dir.join("docs/readme.txt").exists());
        // check flattened file is at root
        assert!(target_dir.join("should_be_flat.txt").exists());
        Ok(())
    }

    #[test]
    fn rm_dir_file() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let temp_dir = temp_dir.path();

        let nested_dir = temp_dir.join("nested_dir");
        fs::create_dir_all(&nested_dir)?;

        let empty_file = temp_dir.join("empty.txt");
        fs::write(&empty_file, "empty")?;

        // assert test file is present
        assert!(empty_file.exists());
        // clear directory
        assert!(remove_dir_contents(temp_dir).is_ok());
        // assert test file is deleted
        assert!(!empty_file.exists());
        // assert nested dir is deleted
        assert!(!nested_dir.exists());
        Ok(())
    }

    #[test]
    fn copy_files_skips_unwanted() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let source_dir = temp_dir.path().join("source");

        // Create various files and directories to test skipping
        fs::create_dir_all(source_dir.join(".hidden_dir"))?;
        fs::create_dir_all(source_dir.join("__pycache__"))?;
        fs::create_dir_all(source_dir.join("normal_dir"))?;

        fs::write(source_dir.join(".hidden_file.txt"), "hidden")?;
        fs::write(source_dir.join("__pycache__/cached.py"), "cached")?;
        fs::write(source_dir.join(".hidden_dir/secret.txt"), "secret")?;
        fs::write(source_dir.join("normal_dir/normal.txt"), "normal")?;
        fs::write(source_dir.join("visible.txt"), "visible")?;

        let target_dir = temp_dir.path().join("target");

        copy_recursively(source_dir, &target_dir, CopyMode::Preserve).unwrap();

        // Check that unwanted files/dirs are skipped
        assert!(!target_dir.join(".hidden_file.txt").exists());
        assert!(!target_dir.join(".hidden_dir").exists());
        assert!(!target_dir.join("__pycache__").exists());

        // Check that normal files/dirs are copied
        assert!(target_dir.join("visible.txt").exists());
        assert!(target_dir.join("normal_dir/normal.txt").exists());

        Ok(())
    }
}
