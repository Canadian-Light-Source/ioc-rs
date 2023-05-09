use std::{
    fs::{self, File},
    io::{self, BufRead, Write},
    path::Path,
};

use blake2::{Blake2s256, Digest};
use colored::Colorize;
use file_hashing::get_hash_folder;
use log::{debug, error, info, trace, warn};

use crate::{
    ioc::IOC,
    log_macros::{cross, exclaim, tick},
};

/**
 * Calculate directory hash from the staging directory.
 * Save to destination directory / data
*/
pub fn hash_ioc(ioc: &IOC) -> io::Result<()> {
    let hash = get_directory_hash(&ioc.stage);
    trace!("hash: {:?}", hash);
    fs::create_dir_all(&ioc.data)?;
    let mut file = File::create(&ioc.hash_file)?;
    file.write_all(hash.as_bytes())?;
    debug!(
        "{} hash_file {:?} written.",
        tick!(),
        &ioc.hash_file.as_path()
    );
    Ok(())
}

/// obtain directory hash
fn get_directory_hash(dir: impl AsRef<Path>) -> String {
    let mut hash = Blake2s256::new();
    get_hash_folder(dir.as_ref(), &mut hash, 1, |_| {}).unwrap()
}

/// check whether destination has been tempered with
pub fn check_hash(ioc: &IOC, force: &bool) -> Result<String, String> {
    // destination doesn't exist yet, that's fine
    if !ioc.destination.exists() {
        return Ok("destination does not yet exist. No hash expected.".to_string());
    }
    let mut hash = String::from("");
    if let Ok(lines) = read_lines(&ioc.hash_file) {
        if let Ok(stored_hash) = lines.last().unwrap() {
            hash = stored_hash;
        };
    }

    let valid_hash = match hash == get_directory_hash(&ioc.destination) {
        false => {
            return if *force {
                warn!(
                    "{} hash mismatch, overwritten by {}",
                    exclaim!(),
                    "--force".yellow()
                );
                Ok(hash)
            } else {
                error!(
                    "{} --> check destination <{:?}> and use `{} {}` to deploy regardless",
                    cross!(),
                    &ioc.destination.as_path(),
                    "ioc install --force".yellow(),
                    &ioc.name.yellow()
                );
                Err("hash mismatch!".to_string())
            }
        }
        true => {
            info!("{} valid hash for {} |{}|", tick!(), &ioc.name.blue(), hash);
            hash
        }
    };
    Ok(valid_hash)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    use crate::settings::Settings;
    use crate::stage;
    use tempfile::tempdir;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn new_test_ioc<P>(ioc_path: P) -> io::Result<IOC>
    where
        P: AsRef<Path>,
    {
        let temp_dir = tempdir()?;
        let stage_dir = temp_dir.path().join("stage");
        let dest_dir = temp_dir.path().join("dest");
        Ok(IOC::new(ioc_path, stage_dir, dest_dir).unwrap())
    }

    #[test]
    fn test_check_hash_no_dest() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();
        assert!(check_hash(&test_ioc, &false).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_hash_mismatch() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();

        std::fs::create_dir_all(&test_ioc.destination)?;
        std::fs::write(test_ioc.destination.join("file1.txt"), "hash test")?;
        std::fs::create_dir_all(&test_ioc.data)?;
        std::fs::write(test_ioc.data.join("hash"), "c00ffee")?;

        assert!(check_hash(&test_ioc, &false).is_err());
        assert!(check_hash(&test_ioc, &true).is_ok());
        Ok(())
    }

    #[test]
    fn test_hash_directory() {
        let dir = Path::new("./tests/hash_test");
        assert_eq!(
            get_directory_hash(dir),
            "55a81f37ab0965a40965b1e8dcef732bca39eb0ef66170056f586a800acff8ee"
        );
    }

    #[test]
    fn test_hash_ioc() {
        let settings = Settings::build("tests/config/test_hash.toml").unwrap();
        let stage_root = settings.get::<String>("filesystem.stage").unwrap();
        let deploy_root = settings.get::<String>("filesystem.deploy").unwrap();
        let test_ioc = IOC::new(
            Path::new("./tests/UTEST_IOC01"),
            Path::new(&stage_root),
            Path::new(&deploy_root),
        )
        .unwrap();
        stage::ioc_stage(&None, Some(test_ioc.clone()), &settings);

        assert!(check_hash(&test_ioc, &true).is_ok());

        assert!(hash_ioc(&test_ioc).is_ok());
        // the actual check
        assert!(&test_ioc.hash_file.exists());
        assert!(fs::remove_dir_all(test_ioc.data).is_ok());
        assert!(fs::remove_dir_all(test_ioc.stage).is_ok());
    }
}
