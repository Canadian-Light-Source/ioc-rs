use std::{fs, io, path::Path};

use diffy::{create_patch, PatchFormatter};
use log::info;

// fn get_patch()

fn get_patch<P>(original: P, modified: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let org_lines = fs::read_to_string(original).expect("Should have been able to read the file");
    let mod_lines = fs::read_to_string(modified).expect("Should have been able to read the file");

    let patch = create_patch(org_lines.as_str(), mod_lines.as_str());
    let f = PatchFormatter::new().with_color();
    let s = f.fmt_patch(&patch.to_owned()).to_string();
    Ok(s)
}

pub fn diff_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name().into_string().unwrap().starts_with('.') {
            continue;
        }
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            if entry.file_name().into_string().unwrap() == "cfg" {
                diff_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                diff_recursively(entry.path(), destination.as_ref())?; // flatten the structure
            }
        } else {
            let patch = get_patch(destination.as_ref().join(entry.file_name()), entry.path())?;
            if patch.lines().count() > 3 {
                info!("===========================================================");
                info!("--- original: {}", entry.path().to_str().unwrap());
                info!(
                    "+++ modified: {}",
                    destination
                        .as_ref()
                        .join(entry.file_name())
                        .to_str()
                        .unwrap()
                );
                info!("DIFF:\n{}", patch);
                info!("===========================================================");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_diff() {
        let original = Path::new("./tests/diff_test/original");
        let modified = Path::new("./tests/diff_test/modified");
        let expected = "\u{1b}[1m--- original\n+++ modified\n\u{1b}[0m\u{1b}[36m@@ -1 +1 @@\u{1b}[0m\n\u{1b}[31m-this is the original version\n\u{1b}[0m\u{1b}[32m+this is the modified version\n\u{1b}[0m";

        let patch = get_patch(original, modified);
        assert_eq!(patch.unwrap(), expected);
    }
}
