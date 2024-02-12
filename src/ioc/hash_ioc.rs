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
    let hash = get_directory_hash(&ioc.destination)?;
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
fn get_directory_hash(dir: impl AsRef<Path>) -> io::Result<String> {
    let mut hash = Blake2s256::new();
    let dir_hash = get_hash_folder(dir.as_ref(), &mut hash, 1, |_| {});
    dir_hash
}

/// check whether destination has been tempered with
pub fn check_hash(ioc: &IOC, force: &bool) -> io::Result<String> {
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

    let destination_hash = get_directory_hash(&ioc.destination)?;
    let valid_hash = match hash == destination_hash {
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
                Err(io::Error::new(io::ErrorKind::InvalidData, "hash mismatch!"))
            };
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
    use crate::test_utils::new_test_ioc;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    // pass if destination is absent, aka, new IOC -> no hash to compare
    #[test]
    fn test_check_hash_no_dest() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();
        assert!(check_hash(&test_ioc, &false).is_ok());
        Ok(())
    }
    // full path tree exists, bogus hash: (force) -> Ok, (no force) -> Fail
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
    // full path tree exists, proper hash: (no force) -> Ok
    #[test]
    fn test_check_hash_match() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();

        std::fs::create_dir_all(&test_ioc.destination)?;
        std::fs::write(test_ioc.destination.join("file1.txt"), "hash test")?;
        std::fs::create_dir_all(&test_ioc.data)?;
        std::fs::write(
            test_ioc.data.join("hash"),
            "72b285e4d6d34b4c8dc8ec1050b125d73f12f95693a744b48e525b738d0d20fe",
        )?;

        assert!(check_hash(&test_ioc, &false).is_ok());
        Ok(())
    }
    // validate proper hashing against known hash
    #[test]
    fn test_hash_directory() {
        let dir = Path::new("./tests/hash_test");
        assert_eq!(
            get_directory_hash(dir).unwrap(),
            "55a81f37ab0965a40965b1e8dcef732bca39eb0ef66170056f586a800acff8ee"
        );
    }
    // check for hash file creation
    #[test]
    fn test_hash_ioc() -> io::Result<()> {
        let test_ioc = new_test_ioc("./tests/UTEST_IOC01").unwrap();
        std::fs::create_dir_all(&test_ioc.data)?;
        std::fs::create_dir_all(&test_ioc.stage)?;
        std::fs::create_dir_all(&test_ioc.destination)?;
        std::fs::write(test_ioc.destination.join("file1.txt"), "hash test")?;

        assert!(hash_ioc(&test_ioc).is_ok());
        // the actual check
        assert!(&test_ioc.hash_file.exists());
        Ok(())
    }
}
