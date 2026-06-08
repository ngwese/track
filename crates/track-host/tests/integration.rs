use std::path::PathBuf;
use std::process::Command;

fn track_bin() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_BIN_EXE_track").expect("CARGO_BIN_EXE_track"))
}

fn wasm_component() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-wasip2/debug/track_cli.wasm")
}

fn fixture_kitchen() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/kitchen")
}

fn run_track(args: &[&str]) -> std::process::Output {
    let mut command = Command::new(track_bin());
    command.env("TRACK_CLI_COMPONENT", wasm_component());
    command.args(args);
    command.output().expect("spawn track")
}

#[test]
fn version_command_succeeds() {
    let output = run_track(&["version"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
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
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn push_without_project_fails_before_guest() {
    let output = run_track(&["push"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires a project"));
}
