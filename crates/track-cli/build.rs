fn main() {
    println!("cargo:rerun-if-changed=../../wit/deps.toml");
    println!("cargo:rerun-if-changed=../../wit/deps.lock");
}
