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
    user_state_base_from(config_base, dirs::state_dir())
}

fn user_state_base_from(config_base: &Path, state_dir: Option<PathBuf>) -> PathBuf {
    state_dir
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
        let state = user_state_base_from(&config, None);
        assert_eq!(state, PathBuf::from("/tmp/track-config/state"));
    }

    #[test]
    fn state_under_platform_state_dir_when_present() {
        let config = PathBuf::from("/tmp/track-config");
        let state = user_state_base_from(&config, Some(PathBuf::from("/var/state")));
        assert_eq!(state, PathBuf::from("/var/state/track"));
    }
}
