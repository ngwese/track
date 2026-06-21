//! CLI integration tests.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn track_cmd() -> Command {
    Command::cargo_bin("track").unwrap()
}

#[test]
fn init_and_validate_standalone_project() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("user-config");
    let state = dir.path().join("user-state");
    let cache = dir.path().join("user-cache");

    track_cmd()
        .current_dir(dir.path())
        .env("TRACK_USER_CONFIG_DIR", &config)
        .env("TRACK_USER_STATE_DIR", &state)
        .env("TRACK_USER_CACHE_DIR", &cache)
        .args(["init", "KITCHEN", "--standalone"])
        .assert()
        .success();

    assert!(config.join("config.json").exists());
    assert!(state.join("node.json").exists());
    assert!(dir.path().join("track.yaml").exists());

    track_cmd()
        .current_dir(dir.path())
        .env("TRACK_USER_CONFIG_DIR", &config)
        .env("TRACK_USER_STATE_DIR", &state)
        .env("TRACK_USER_CACHE_DIR", &cache)
        .args(["schema", "validate"])
        .assert()
        .success();
}

#[test]
fn push_dry_run_plans_schema_event() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("user-config");
    let state = dir.path().join("user-state");
    let cache = dir.path().join("user-cache");

    track_cmd()
        .current_dir(dir.path())
        .env("TRACK_USER_CONFIG_DIR", &config)
        .env("TRACK_USER_STATE_DIR", &state)
        .env("TRACK_USER_CACHE_DIR", &cache)
        .args(["init", "API", "--standalone"])
        .assert()
        .success();

    track_cmd()
        .current_dir(dir.path())
        .env("TRACK_USER_CONFIG_DIR", &config)
        .env("TRACK_USER_STATE_DIR", &state)
        .env("TRACK_USER_CACHE_DIR", &cache)
        .args(["push", "--dry-run", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("schema.init"));
}

#[test]
fn init_refuses_existing_without_force() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("track.yaml"), "type: project\n").unwrap();

    track_cmd()
        .current_dir(dir.path())
        .env("TRACK_USER_CONFIG_DIR", dir.path().join("cfg"))
        .env("TRACK_USER_STATE_DIR", dir.path().join("st"))
        .env("TRACK_USER_CACHE_DIR", dir.path().join("ca"))
        .args(["init", "X", "--standalone"])
        .assert()
        .failure();
}
