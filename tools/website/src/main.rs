use std::{fs, path::Path};

use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    let source_dir = Path::new("website");
    let target_dir = Path::new("website-output");

    copy_website_source_to_target_dir(source_dir, target_dir)?;

    Ok(())
}

fn copy_website_source_to_target_dir(
    source_dir: &Path,
    target_dir: &Path,
) -> anyhow::Result<()> {
    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }

        let source_path = entry.path();
        let target_path = {
            let path_within_source_dir =
                entry.path().strip_prefix(source_dir)?;
            target_dir.join(path_within_source_dir)
        };

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source_path, target_path)?;
    }

    Ok(())
}
