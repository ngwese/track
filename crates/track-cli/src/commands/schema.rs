use crate::output;
use crate::track::host::session;
use track_types::SchemaValidateResponse;

pub fn validate(
    invocation: &session::Invocation,
    json: bool,
) -> Result<(), ()> {
    let manifest = invocation.manifest_path.clone();
    let valid = manifest.is_some();
    let message = if valid {
        "project manifest discovered; schema validation stub passed"
    } else {
        "no project manifest in scope"
    };

    if json {
        output::print_json(&SchemaValidateResponse {
            valid,
            manifest,
        });
    } else {
        output::print_text(message);
    }

    if valid {
        Ok(())
    } else {
        output::print_error(json, message, 2)
    }
}
