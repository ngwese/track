use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
struct VersionFile {
    version: Option<String>,
    digest: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct VersionConfig {
    pub version: Option<String>,
    pub digest: Option<String>,
}

pub fn read(path: &Path) -> VersionConfig {
    let Ok(text) = fs::read_to_string(path) else {
        return VersionConfig::default();
    };
    let Ok(file) = serde_yaml::from_str::<VersionFile>(&text) else {
        return VersionConfig::default();
    };
    VersionConfig {
        version: file.version,
        digest: file.digest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn reads_version_and_digest() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("track-version.yaml");
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "version: \"0.2.0\"").unwrap();
        writeln!(file, "digest: \"abc123\"").unwrap();

        let config = read(&path);
        assert_eq!(config.version.as_deref(), Some("0.2.0"));
        assert_eq!(config.digest.as_deref(), Some("abc123"));
    }
}
