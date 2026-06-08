use crate::paths;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use track_host_wit::track::host::registry::{self, Error, ErrorCode};

pub fn resolve(version: &str, expected_digest: Option<&str>) -> Result<registry::Artifact, Error> {
    let cache_file = component_cache_path(version);
    if !cache_file.is_file() {
        let source = locate_source_artifact()?;
        fs::create_dir_all(cache_file.parent().expect("cache parent")).map_err(io_error)?;
        fs::copy(&source, &cache_file).map_err(io_error)?;
    }

    let digest = file_digest(&cache_file)?;
    if let Some(expected) = expected_digest {
        if expected != digest {
            return Err(Error {
                code: ErrorCode::DigestMismatch,
                message: format!("expected digest {expected}, found {digest}"),
            });
        }
    }

    Ok(registry::Artifact {
        version: version.into(),
        digest,
        cache_path: format!("components/{version}/track_cli.wasm"),
    })
}

pub fn runtime_component_path(
    version: &str,
    expected_digest: Option<&str>,
) -> Result<PathBuf, Error> {
    resolve(version, expected_digest)?;
    Ok(component_cache_path(version))
}

fn component_cache_path(version: &str) -> PathBuf {
    paths::user_cache_dir()
        .join("components")
        .join(version)
        .join("track_cli.wasm")
}

fn locate_source_artifact() -> Result<PathBuf, Error> {
    if let Ok(path) = env::var("TRACK_CLI_COMPONENT") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(Error {
            code: ErrorCode::NotFound,
            message: format!("TRACK_CLI_COMPONENT not found: {}", path.display()),
        });
    }

    let mut candidates = Vec::new();
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("track_cli.wasm"));
            candidates.push(dir.join("../wasm32-wasip2/debug/track_cli.wasm"));
            candidates.push(dir.join("../wasm32-wasip2/release/track_cli.wasm"));
        }
    }
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let root = PathBuf::from(manifest_dir);
        candidates.push(root.join("../../target/wasm32-wasip2/debug/track_cli.wasm"));
        candidates.push(root.join("../../target/wasm32-wasip2/release/track_cli.wasm"));
    }

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(Error {
        code: ErrorCode::NotFound,
        message: "track-cli component not found; build with \
                  `cargo build -p track-cli --target wasm32-wasip2` or set TRACK_CLI_COMPONENT"
            .into(),
    })
}

fn file_digest(path: &Path) -> Result<String, Error> {
    let mut file = fs::File::open(path).map_err(io_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(io_error)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn io_error(err: impl std::fmt::Display) -> Error {
    Error {
        code: ErrorCode::NotFound,
        message: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn digest_is_stable() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sample.wasm");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"wasm").unwrap();
        let digest = file_digest(&path).unwrap();
        assert_eq!(digest.len(), 64);
        assert_eq!(digest, file_digest(&path).unwrap());
    }
}
