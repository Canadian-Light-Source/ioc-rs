use std::path::Path;

use glob::glob;

// logging
use crate::log_macros::{exclaim, tick};
use colored::Colorize;
use log::trace;

/// Customized Conda environment folder
pub const CONDA_ENV_DIR: &str = "env";

/// Customized Conda environment yaml description.
pub const CONDA_ENV_CFG: &str = "conda_config.yaml";

/// Check if the directory contains Python files, indicating it's a Python IOC
pub fn is_python_ioc(dir: impl AsRef<Path>) -> bool {
    let pattern = format!("{}/**/*.py", dir.as_ref().display());

    // Use glob to find Python files
    if let Ok(entries) = glob(&pattern) {
        let has_python_files = entries.count() > 0;
        if has_python_files {
            trace!("{} Found Python files in directory", tick!());
        } else {
            trace!("{} No Python files found", exclaim!());
        }
        has_python_files
    } else {
        trace!("{} Failed to search for Python files", exclaim!());
        false
    }
}
