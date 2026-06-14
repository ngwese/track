//! Execution telemetry exclusion policy (SRD §2.15).

/// Determines which reduced state is eligible for YAML export.
pub trait YamlExclusionPolicy {
    /// Returns true when execution claim/progress events should appear in YAML.
    fn includes_execution_events(&self) -> bool;
}

/// Default policy: execution telemetry never materializes to YAML.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DefaultYamlExclusionPolicy;

impl YamlExclusionPolicy for DefaultYamlExclusionPolicy {
    fn includes_execution_events(&self) -> bool {
        false
    }
}
