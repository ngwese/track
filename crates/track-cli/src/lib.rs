mod commands;
mod output;
mod router;

wit_bindgen::generate!({
    world: "cli-guest",
    path: "../../wit/track",
});

struct TrackCli;

impl exports::wasi::cli::run::Guest for TrackCli {
    fn run() -> Result<(), ()> {
        router::run()
    }
}

export!(TrackCli);
