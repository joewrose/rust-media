use std::path::Path;

use log::warn;

fn walk_dir(
    dir: &Path,
    file_paths: &mut Vec<String>,
    recursive: bool,
) -> Result<Vec<String>, std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let file_path = entry.path();

        if recursive && file_path.is_dir() {
            walk_dir(&file_path, file_paths, recursive)?;
        } else if file_path.extension().and_then(std::ffi::OsStr::to_str) == Some("mp3") {
            // This could be a lossy conversion. Read the docs about this
            file_paths.push(file_path.display().to_string());
        } else {
            warn!(
                "Found non-mp3 file or directory {}",
                file_path.display().to_string()
            );
        }
    }
    // Should we be converting to a vec here?
    Ok(file_paths.to_vec())
}

pub(crate) fn get_file_paths(dir: &Path, recursive: bool) -> Result<Vec<String>, std::io::Error> {
    match dir.is_dir() {
        // Return a vector containing all audio files in the dir
        true => {
            let mut file_paths: Vec<String> = Vec::new();
            walk_dir(dir, &mut file_paths, recursive)
        }
        // Return a vector of size one
        false => Ok(Vec::from([dir.display().to_string()])),
    }
}
