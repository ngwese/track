pub fn init(level: &str) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level))
        .format_target(true)
        .init();
}
