wit_bindgen::generate!({
    world: "cli-guest",
    path: "../../wit/track",
});

struct TrackCli;

impl exports::wasi::cli::run::Guest for TrackCli {
    fn run() -> Result<(), ()> {
        let invocation = track::host::session::get();
        let caps = track::host::capabilities::get();
        let areas = track::host::locations::list_available();

        println!("track-cli {} (stub)", invocation.tool_version);
        println!("argv: {:?}", invocation.argv);
        if let Some(root) = &invocation.project_root {
            println!("project-root: {root}");
        }
        if let Some(manifest) = &invocation.manifest_path {
            println!("manifest: {manifest}");
        }
        println!(
            "capabilities: network={} stdout={}",
            caps.network, caps.stdout
        );
        println!("areas: {:?}", areas);

        if invocation.argv.len() > 1 && invocation.argv[1] == "interfaces" {
            exercise_imports()?;
        }

        Ok(())
    }
}

fn exercise_imports() -> Result<(), ()> {
    let _ = track::host::auth::list_workspaces();
    let _ = track::host::user_config::read();
    let _ = track::host::project_state::read();
    let _ = track::host::offline_queue::list_queued(None);
    let _ = track::host::registry::resolve("0.1.0", None);
    Ok(())
}

export!(TrackCli);
