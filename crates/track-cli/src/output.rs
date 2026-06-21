//! Human and JSON stdout rendering.

use track_node::{InitResponse, PushResponse, SchemaValidateResponse};

/// Print init outcome.
pub fn print_init(json: bool, response: &InitResponse) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(response).expect("serialize init response")
        );
        return;
    }
    println!("Initialized project {}", response.key);
    println!("  root: {}", response.project_root.display());
    println!("  project_uuid: {}", response.project_uuid);
}

/// Print schema validate outcome.
pub fn print_schema_validate(json: bool, response: &SchemaValidateResponse) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(response).expect("serialize validate response")
        );
        return;
    }
    if response.valid {
        println!("Schema is valid.");
        return;
    }
    eprintln!("Schema validation failed:");
    for err in &response.errors {
        eprintln!("  {}:{}: {}", err.file, err.path, err.message);
    }
}

/// Print push outcome.
pub fn print_push(json: bool, response: &PushResponse) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(response).expect("serialize push response")
        );
        return;
    }
    if response.dry_run {
        println!(
            "Dry run: would push {} event(s) (schema: {}, work: {})",
            response.summary.event_count,
            response.summary.schema_count,
            response.summary.work_count
        );
    }
}
