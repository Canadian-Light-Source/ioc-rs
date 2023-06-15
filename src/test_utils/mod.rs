use crate::ioc::IOC;
use std::io;
use std::path::Path;
use tempfile::tempdir;

#[cfg(test)]
pub fn new_test_ioc<P>(ioc_path: P) -> io::Result<IOC>
where
    P: AsRef<Path>,
{
    let temp_dir = tempdir()?;
    let stage_dir = temp_dir.path().join("stage");
    let dest_dir = temp_dir.path().join("dest");
    let template_dir = temp_dir.path().join("templates");

    Ok(IOC::new(ioc_path, stage_dir, dest_dir, template_dir).expect("failed to build IOC!"))
}
