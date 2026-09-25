mod etc;
use anyhow::anyhow;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateVnPaths {
    pub game: PathBuf,
    pub context_narrative: Vec<PathBuf>,
    pub context_dialogue: Vec<PathBuf>,
}

impl TemplateVnPaths {
    pub(crate) fn context_narrative(&self) -> &[PathBuf] {
        &self.context_narrative
    }

    pub(crate) fn context_dialogue(&self) -> &[PathBuf] {
        &self.context_dialogue
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct TemplateVnMetadata {
    pub name: String,
    pub stylized_name: String,
    pub version: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateVn {
    metadata: TemplateVnMetadata,
    paths: TemplateVnPaths,
}

impl TemplateVn {
    pub(crate) fn load(root: &Path) -> anyhow::Result<Self> {
        let manifest = fs::read_to_string(root.join("Novelize.toml"))?;
        let template: Self = manifest.parse()?;
        template.validate_at(root)?;
        Ok(template)
    }

    pub(crate) fn paths(&self) -> &TemplateVnPaths {
        &self.paths
    }

    fn validate_at(&self, root: &Path) -> anyhow::Result<()> {
        etc::is_path_relative_and_bounded(&self.paths.game)?;
        let game_path = root.join(&self.paths.game);
        if !game_path.exists() {
            return Err(anyhow!("path `{}` does not exist", game_path.display()));
        }
        if !game_path.is_dir() {
            return Err(anyhow!("path `{}` is a file, and not a directory when it's expected to be the game directory", self.paths.game.display()));
        }

        for path in self
            .paths
            .context_narrative
            .iter()
            .chain(self.paths.context_dialogue.iter())
        {
            etc::is_path_relative_and_bounded(path)?;
            let context_path = root.join(path);
            if context_path.is_dir() {
                return Err(anyhow!("path `{}` is a directory, and not a file when it's expected to be context", path.display()));
            }
            if !context_path.exists() {
                return Err(anyhow!("path `{}` does not exist", context_path.display()));
            }
        }

        Ok(())
    }

    pub fn is_game_valid(
        &self,
        root: &Path,
        sdk_path: PathBuf,
    ) -> Result<(), crate::renpy::RenPyError> {
        crate::renpy::is_valid_renpy_sdk(sdk_path.clone())?;
        let game_path = root
            .join(&self.paths.game)
            .canonicalize()
            .map_err(crate::renpy::RenPyError::Io)?;
        let lint_output = crate::renpy::run_renpy_lint(&sdk_path, &game_path)?;
        if !lint_output.status.success() {
            return Err(crate::renpy::RenPyError::LintFailed(lint_output));
        }
        Ok(())
    }
}

impl FromStr for TemplateVn {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self, anyhow::Error> {
        let template_vn: Self = toml::from_str(str)?;
        Ok(template_vn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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
context-narrative = ["context-narrative.txt"]
context-dialogue = ["context-dialogue.txt"]"#;

        let _template_vn_manifest: TemplateVn = toml::from_str(&manifest)?;

        Ok(())
    }

    #[test]
    fn loads_split_context_from_template_root() -> anyhow::Result<()> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("novelize-context-test-{unique}"));
        fs::create_dir_all(root.join("game"))?;
        fs::write(
            root.join("Novelize.toml"),
            "[metadata]\nname = \"test\"\nstylized-name = \"Test\"\nversion = \"0.1.0\"\n\n[paths]\ngame = \"game\"\ncontext-narrative = [\"narrative.txt\"]\ncontext-dialogue = [\"dialogue.txt\"]\n",
        )?;
        fs::write(root.join("narrative.txt"), "narrative")?;
        fs::write(root.join("dialogue.txt"), "dialogue")?;

        let template = TemplateVn::load(&root)?;

        assert_eq!(template.paths().context_narrative(), [PathBuf::from("narrative.txt")]);
        assert_eq!(template.paths().context_dialogue(), [PathBuf::from("dialogue.txt")]);

        fs::remove_dir_all(root)?;
        Ok(())
    }
}