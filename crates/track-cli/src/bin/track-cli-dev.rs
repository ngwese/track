//! Native dev entrypoint for the track-cli router (TRACK_DEV_NATIVE workflow).
//! Does not implement host WIT imports — use only for router unit testing.

fn main() {
    eprintln!("track-cli-dev requires TRACK_DEV_NATIVE host wiring (use `make run` for full stack)");
    std::process::exit(2);
}
