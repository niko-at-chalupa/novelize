mod etc;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateVnPaths {
    pub game: PathBuf,
    pub context: Vec<PathBuf>,
}

impl TemplateVnPaths {
    pub(crate) fn game(&self) -> &Path {
        &self.game
    }

    pub(crate) fn context(&self) -> &[PathBuf] {
        &self.context
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateVnMetadata {
    pub name: String,
    pub stylized_name: String,
    pub version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateVn {
    metadata: TemplateVnMetadata,
    paths: TemplateVnPaths,
}

impl TemplateVn {
    fn validate(&self) -> anyhow::Result<()> {
        etc::is_path_relative_and_bounded(&self.paths.game)?;
        if !&self.paths.game.is_dir() {
            return Err(anyhow!("path `{}` is a file, and not a directory when it's expected to be the game directory", &self.paths.game.display()));
        }
        if !&self.paths.game.exists() {
            return Err(anyhow!("path `{}` does not exist", &self.paths.game.display()))
        }
        
        for path in &self.paths.context {
            etc::is_path_relative_and_bounded(path)?;
            if path.is_dir() {
                return Err(anyhow!("path `{}` is a directory, and not a file when it's expected to be context", path.display()));
            }
            if !path.exists() {
                return Err(anyhow!("path `{}` does not exist", &self.paths.game.display()))
            }
        }

        Ok(())
    }

    pub fn is_game_valid(&self, sdk_path: PathBuf) -> Result<(), crate::renpy::RenPyError> {
        crate::renpy::is_valid_renpy_sdk(sdk_path.clone())?;
        let lint_output = crate::renpy::run_renpy_lint(&sdk_path, &self.paths.game)?;
        if !lint_output.status.success() {
            return Err(crate::renpy::RenPyError::LintFailed(lint_output).into());
        }
        Ok(())
    }
}

impl FromStr for TemplateVn {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self, anyhow::Error> {
        let template_vn: Self = toml::from_str(&str)?;
        template_vn.validate()?;
        Ok(template_vn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializing() -> anyhow::Result<()> {
        let manifest = r#"[metadata]
name = "novelize"
stylized-name = "Novelize"
version = "0.1.0"

# This lists out _RELATIVE_ paths to things. Everything's required. No path can
# go underneath this directory.
[paths]
game = "game"
# Loaded into the model's context when generating dialogue or narrative.
context = [
    "character_info.txt",
]"#;

        let template_vn_manifest: TemplateVn = toml::from_str(&manifest)?;

        Ok(())
    }
}