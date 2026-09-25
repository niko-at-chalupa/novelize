use anyhow::anyhow;
use std::path::{Component, Path};

pub(super) fn is_path_relative_and_bounded(path: &Path) -> anyhow::Result<()> {
    if path.is_absolute() {
        return Err(anyhow!("path `{}` must be relative, not absolute", path.display()));
    }

    let mut depth = 0;
    for component in path.components() {
        match component {
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(anyhow!(
                        "path `{}` attempts to escape the parent directory using `..`",
                        path.display()
                    ));
                }
            }
            Component::Normal(_) => {
                depth += 1;
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir => {
                return Err(anyhow!("path `{}` contains invalid root components", path.display()));
            }
        }
    }

    Ok(())
}