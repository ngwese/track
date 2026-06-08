use serde::Serialize;
use track_types::ErrorResponse;

pub fn print_json<T: Serialize>(value: &T) {
    println!("{}", serde_json::to_string(value).expect("serialize json"));
}

pub fn print_text(message: &str) {
    println!("{message}");
}

pub fn print_error(json: bool, message: impl Into<String>, code: u8) -> Result<(), ()> {
    let message = message.into();
    if json {
        print_json(&ErrorResponse {
            error: message,
            code,
        });
    } else {
        eprintln!("error: {message}");
    }
    Err(())
}
