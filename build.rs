use std::{env, process::Command};

fn main() {
    Command::new("node")
        .arg("build.js")
        .env("OUT_DIR", env::var("OUT_DIR").unwrap())
        .output()
        .unwrap();
}
