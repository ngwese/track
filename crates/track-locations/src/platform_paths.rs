//! OS-specific base paths for user buckets.

use std::path::{Path, PathBuf};

use crate::LocationError;

/// Resolve the user-config bucket root (`…/track/` under the platform config dir).
pub fn user_config_base() -> Result<PathBuf, LocationError> {
    dirs::config_dir()
        .map(|p| p.join("track"))
        .ok_or_else(|| LocationError::PlatformUnavailable("config_dir".into()))
}

/// Resolve the user-state bucket root.
pub fn user_state_base(config_base: &Path) -> PathBuf {
    dirs::state_dir()
        .map(|p| p.join("track"))
        .unwrap_or_else(|| config_base.join("state"))
}

/// Resolve the user-cache bucket root.
pub fn user_cache_base() -> Result<PathBuf, LocationError> {
    dirs::cache_dir()
        .map(|p| p.join("track"))
        .ok_or_else(|| LocationError::PlatformUnavailable("cache_dir".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_fallback_under_config_when_no_state_dir() {
        let config = PathBuf::from("/tmp/track-config");
        let state = user_state_base(&config);
        assert_eq!(state, PathBuf::from("/tmp/track-config/state"));
    }
}
