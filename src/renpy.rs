use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::fmt::self;

#[derive(Debug)]
#[allow(dead_code)]
pub enum RenPyError {
    SdkNotFound,
    SdkInvalid(PathBuf),
    LintFailed(Output),
    Io(std::io::Error),
    LintSpawnFailed(std::io::Error),
}

impl Error for RenPyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::LintSpawnFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for RenPyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SdkNotFound => write!(f, "Ren'Py SDK not found"),
            Self::SdkInvalid(p) => write!(f, "Ren'Py SDK {} is invalid", p.display()),
            Self::LintFailed(o) => write!(f, "Ren'Py lint failed:\n{}", String::from_utf8_lossy(&o.stderr)),
            Self::Io(e) => write!(f, "{}", e),
            Self::LintSpawnFailed(e) => write!(f, "Failed to spawn process for Ren'Py lint:\n{}", e),
        }
    }
}

pub fn is_valid_renpy_sdk(sdk_path: PathBuf) -> Result<(), RenPyError> {
    if !sdk_path.is_dir() {
        return Err(RenPyError::SdkInvalid(sdk_path));
    }

    let renpy_py = sdk_path.join("renpy.py");
    let lib_dir = sdk_path.join("lib");

    if renpy_py.is_file() && lib_dir.is_dir() {
        Ok(())
    } else {
        Err(RenPyError::SdkInvalid(sdk_path))
    }
}

pub fn run_renpy_lint(sdk_path: &Path, project_path: &Path) -> Result<Output, RenPyError> {
    let os = std::env::consts::OS;
    let renpy_executable: &Path = {
        if os == "windows" {
            &sdk_path.join("renpy.exe")
        } else {
            &sdk_path.join("renpy.sh")
        }
    };

    if !project_path.exists() {
        return Err(RenPyError::Io(std::io::ErrorKind::NotFound.into()));
    }

    Command::new(renpy_executable)
        .arg(project_path)
        .arg("lint")
        .env("SDL_AUDIODRIVER", "dummy")
        .env("SDL_VIDEODRIVER", "dummy")
        .output()
        .map_err(RenPyError::LintSpawnFailed)
}
