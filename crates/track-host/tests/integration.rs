use std::path::PathBuf;
use std::process::Command;

fn track_bin() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_BIN_EXE_track").expect("CARGO_BIN_EXE_track"))
}

fn wasm_component() -> PathBuf {
    let track_exe = track_bin();
    track_exe
        .parent()
        .expect("track exe parent")
        .parent()
        .expect("target directory")
        .join("wasm32-wasip2/debug/track_cli.wasm")
}

fn fixture_kitchen() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/kitchen")
}

fn run_track(args: &[&str]) -> std::process::Output {
    let mut command = Command::new(track_bin());
    command.env("TRACK_CLI_COMPONENT", wasm_component());
    command.env("TRACK_LOG_LEVEL", "warn");
    command.args(args);
    command.output().expect("spawn track")
}

#[test]
fn version_command_succeeds() {
    let output = run_track(&["version"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("track-cli"));
}

#[test]
fn json_version_emits_json() {
    let output = run_track(&["--json", "version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim_start().starts_with('{'));
}

#[test]
fn schema_validate_in_fixture_project() {
    let fixture = fixture_kitchen();
    let output = run_track(&[
        "--project",
        fixture.to_str().expect("utf8 path"),
        "schema",
        "validate",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn help_lists_host_and_guest_options() {
    let output = run_track(&["help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Host options:"));
    assert!(stdout.contains("--project"));
    assert!(stdout.contains("Guest options:"));
}

#[test]
fn track_project_env_overrides_discovery() {
    let fixture = fixture_kitchen();
    let expected = std::fs::canonicalize(&fixture).expect("canonicalize fixture");
    let output = Command::new(track_bin())
        .env("TRACK_CLI_COMPONENT", wasm_component())
        .env("TRACK_LOG_LEVEL", "warn")
        .env("TRACK_PROJECT", &expected)
        .args(["interfaces"])
        .output()
        .expect("spawn track");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("project-root: {}", expected.display())));
}

#[test]
fn push_without_project_reaches_guest() {
    let output = run_track(&["push"]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains("requires a project"));
}
