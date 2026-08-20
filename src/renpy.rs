use std::process::{Command, ExitStatus};
use std::path::Path;

pub fn is_valid_renpy_sdk(sdk_path: &Path) -> bool {
    if !sdk_path.is_dir() {
        return false;
    }

    let renpy_py = sdk_path.join("renpy.py");
    let lib_dir = sdk_path.join("lib");

    renpy_py.is_file() && lib_dir.is_dir()
}

pub fn run_renpy_lint(sdk_path: &Path, project_path: &Path) -> std::io::Result<ExitStatus> {
    let os = std::env::consts::OS;
    let renpy_executable: &Path = {
        if os == "windows" {
            &project_path.join("renpy.exe")
        } else {
            &project_path.join("renpy.sh")
        }
    };

    if !project_path.exists() {
        return Err(std::io::ErrorKind::NotFound.into());
    }

    Command::new(renpy_executable)
        .arg(project_path)
        .arg("lint")
        .env("SDL_AUDIODRIVER", "dummy")
        .env("SDL_VIDEODRIVER", "dummy")
        .status()
}