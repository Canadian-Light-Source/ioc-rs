use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use blake2::{Blake2s256, Digest};
use colored::Colorize;
use file_hashing::get_hash_folder;
use log::{debug, trace};

use crate::{ioc::IOC, log_macros::tick};

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
