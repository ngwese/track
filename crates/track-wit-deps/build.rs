fn main() -> Result<(), Box<dyn std::error::Error>> {
    wit_deps::lock_sync!("../../wit")?;
    println!("cargo:rerun-if-changed=../../wit/deps.toml");
    println!("cargo:rerun-if-changed=../../wit/deps.lock");
    Ok(())
}
