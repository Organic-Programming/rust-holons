//! Parse HOLON.md identity files.

use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Parsed identity from a HOLON.md file.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HolonIdentity {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub given_name: String,
    #[serde(default)]
    pub family_name: String,
    #[serde(default)]
    pub motto: String,
    #[serde(default)]
    pub composer: String,
    #[serde(default)]
    pub clade: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub born: String,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub reproduction: String,
    #[serde(default)]
    pub generated_by: String,
    #[serde(default)]
    pub proto_status: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Parse a HOLON.md file and return its identity.
pub fn parse_holon(path: &Path) -> Result<HolonIdentity, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;

    if !text.starts_with("---") {
        return Err(format!("{}: missing YAML frontmatter", path.display()).into());
    }

    let end = text[3..]
        .find("---")
        .ok_or_else(|| format!("{}: unterminated YAML frontmatter", path.display()))?;

    let frontmatter = &text[3..3 + end].trim();
    let identity: HolonIdentity = serde_yaml::from_str(frontmatter)?;
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_holon() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_holon_rust.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(
            f,
            "---\nuuid: \"abc-123\"\ngiven_name: \"test\"\nfamily_name: \"Test\"\n\
             motto: \"A test.\"\nclade: \"deterministic/pure\"\nlang: \"rust\"\n---\n# test"
        )
        .unwrap();

        let id = parse_holon(&path).unwrap();
        assert_eq!(id.uuid, "abc-123");
        assert_eq!(id.given_name, "test");
        assert_eq!(id.lang, "rust");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let dir = std::env::temp_dir();
        let path = dir.join("no_fm_rust.md");
        fs::write(&path, "# No frontmatter\n").unwrap();
        assert!(parse_holon(&path).is_err());
        fs::remove_file(&path).unwrap();
    }
}
