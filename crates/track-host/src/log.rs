pub fn trace(message: impl AsRef<str>) {
    if std::env::var_os("TRACK_LOG").is_some() {
        eprintln!("[track-host] {}", message.as_ref());
    }
}
