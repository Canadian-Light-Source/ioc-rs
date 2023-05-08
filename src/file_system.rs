use std::path::Path;
use std::{fs, io};

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
pub fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name().into_string().unwrap().starts_with('.') {
            continue;
        }
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            if entry.file_name().into_string().unwrap() == "cfg" {
                copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                copy_recursively(entry.path(), destination.as_ref())?; // flatten the structure
            }
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn copy_files() {

        let source_dir = Path::new("tests/filesystem_test/dummy_files/");
        let target_dir = Path::new("tests/filesystem_test/copy_recursive/");

        let empty_file = target_dir.join("empty.txt");
        let nested_file = target_dir.join("nested_file.txt");
        let cfg_file = target_dir.join("cfg/file_in_cfg_dir.txt");

        copy_recursively(source_dir, target_dir).unwrap();

        // check if all files made it
        assert!(empty_file.exists());
        assert!(cfg_file.exists());
        assert!(fs::remove_dir_all(target_dir).is_ok());
    }

    #[test]
    fn rm_dir_file() {

        let empty_file = Path::new("tests/filesystem_test/file/empty.txt");
        let test_dir = empty_file.parent().unwrap();
        if !test_dir.exists() {
            fs::create_dir(test_dir).expect("cannot create test directory");
        }
        fs::File::create(empty_file).expect("create failed");

        // assert test file is present
        assert!(empty_file.exists());
        // clear directory
        assert!(remove_dir_contents(test_dir).is_ok());
        // assert test file is deleted
        assert!(!empty_file.exists());
    }

    #[test]
    fn rm_dir_directory() {

        let test_root = Path::new("tests/filesystem_test/directory/");

        let empty_file_sub_1 = test_root.join("sub_dir_1/empty.txt");
        let test_dir_1 = empty_file_sub_1.parent().unwrap();
        fs::create_dir_all(test_dir_1).unwrap();
        fs::File::create(empty_file_sub_1.as_path()).expect("create failed");
        assert!(empty_file_sub_1.exists());

        let empty_file_sub_2 = test_root.join("sub_dir_2/empty.txt");
        let test_dir_2 = empty_file_sub_2.parent().unwrap();
        fs::create_dir_all(test_dir_2).unwrap();
        fs::File::create(empty_file_sub_2.as_path()).expect("create failed");
        assert!(empty_file_sub_2.exists());

        // clear directory contents recursively
        assert!(remove_dir_contents(test_root).is_ok());
        // assert root dir is empty
        assert!(test_root.read_dir().unwrap().next().is_none());
    }
}
