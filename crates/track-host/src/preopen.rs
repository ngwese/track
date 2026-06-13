use std::path::Path;
use track_host_wit::track::host::{capabilities, locations::Area};

pub fn preopened_areas(project_root: Option<&Path>) -> Vec<Area> {
    let mut areas = vec![Area::UserConfig, Area::UserState, Area::UserCache];
    if project_root.is_some() {
        areas.extend([Area::ProjectConfig, Area::ProjectState, Area::ProjectCache]);
    }
    areas
}

pub fn capability_flags() -> capabilities::CapabilityFlags {
    capabilities::CapabilityFlags {
        network: true,
        hub_allowlist: Vec::new(),
        stdin: true,
        stdout: true,
        stderr: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_areas_without_project() {
        assert_eq!(
            preopened_areas(None),
            vec![Area::UserConfig, Area::UserState, Area::UserCache,]
        );
    }

    #[test]
    fn includes_project_areas_when_root_present() {
        assert_eq!(
            preopened_areas(Some(Path::new("/proj"))),
            vec![
                Area::UserConfig,
                Area::UserState,
                Area::UserCache,
                Area::ProjectConfig,
                Area::ProjectState,
                Area::ProjectCache,
            ]
        );
    }
}
