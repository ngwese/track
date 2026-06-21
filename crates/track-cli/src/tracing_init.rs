//! stderr tracing subscriber setup.

use tracing_subscriber::EnvFilter;

/// Initialize tracing to stderr.
pub fn init_tracing(verbose: bool, debug: bool, trace: bool) {
    let default_level = resolved_level(verbose, debug, trace);
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();
}

/// Resolve default filter level from CLI flags.
pub fn resolved_level(verbose: bool, debug: bool, trace: bool) -> &'static str {
    if trace {
        "trace"
    } else if debug {
        "debug"
    } else if verbose {
        "info"
    } else {
        "warn"
    }
}
