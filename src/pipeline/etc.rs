use regex::Regex;
use std::fs;
use std::path::Path;

/// Helper to recursively copy directories
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn clean_code_block_wrappers(text: &str) -> String {
    let re = Regex::new(r"(?m)^```[^\r\n]*\r?\n?").unwrap();
    re.replace(text, "")
        .into_owned()
        .trim_end_matches("```")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_code_block_wrappers_test() {
        let content = r#"iajsfiajisfjajfiajsifjija
        asfiajsifjasfjiajsfjasifjaisf
        asfiajisfajifsjiaifjiajsfijs
        asdjasidjijd"#;
        let prefixes = vec!["```renpy", "```json", "```"];
        let end = "```";

        let mut samples: Vec<String> = vec![];

        // {prefix}
        // {content}
        // {end}
        for prefix in prefixes {
            samples.push(format!("{}\n{}\n{}", prefix, content, end));
        }
        // {content}
        samples.push(content.to_string());

        for sample in samples {
            let cleaned = clean_code_block_wrappers(&sample);
            assert_eq!(cleaned, content)
        }
    }
}
