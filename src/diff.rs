use std::{fs, io, path::Path};

use diffy::{create_patch, PatchFormatter};

// fn get_patch()

pub fn get_patch<P>(original: P, modified: P) -> io::Result<String>
// fn get_patch<P>(original: P, modified: P) -> io::Result<Patch<'static, str>>
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
