use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
struct Manifest {
    tool: Option<ToolSection>,
}

#[derive(Debug, Clone, Deserialize)]
struct ToolSection {
    version: Option<String>,
    digest: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ManifestInfo {
    pub tool_version: Option<String>,
    pub tool_digest: Option<String>,
}

pub fn read(manifest_path: &Path) -> ManifestInfo {
    let Ok(text) = fs::read_to_string(manifest_path) else {
        return ManifestInfo::default();
    };
    let Ok(manifest) = serde_yaml::from_str::<Manifest>(&text) else {
        return ManifestInfo::default();
    };
    ManifestInfo {
        tool_version: manifest.tool.as_ref().and_then(|t| t.version.clone()),
        tool_digest: manifest.tool.as_ref().and_then(|t| t.digest.clone()),
    }
}
