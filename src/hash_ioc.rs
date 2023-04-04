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
pub fn hash_ioc(ioc: &IOC) -> std::io::Result<()> {
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
    let directory = dir.as_ref().to_str().unwrap();
    get_hash_folder(directory, &mut hash, 1, |_| {}).unwrap()
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
            if *force {
                warn!(
                    "{} hash mismatch, overwritten by {}",
                    exclaim!(),
                    "--force".yellow()
                );
                return Ok(hash);
            } else {
                error!(
                    "{} --> check destination <{:?}> and use `{} {}` to deploy regardless",
                    cross!(),
                    &ioc.destination.as_path(),
                    "ioc install --force".yellow(),
                    &ioc.name.yellow()
                );
                return Err("hash mismatch!".to_string());
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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
        let test_ioc = IOC::new(
            Path::new("./tests/MTEST_IOC01"),
            Path::new("./tests/tmp/stage/"),
            Path::new("./tests/tmp"),
        )
        .unwrap();
        let _ = hash_ioc(&test_ioc);
        // the actual check
        assert!(&test_ioc.hash_file.exists());
    }
}
