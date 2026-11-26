use browserslist::{Opts, resolve};
use std::env;

fn main() {
    let mut args = env::args();
    args.next();

    let mut opts = Opts::default();
    let mut queries = Vec::new();
    args.for_each(|arg| match &*arg {
        "--mobile-to-desktop" => opts.mobile_to_desktop = true,
        "--ignore-unknown-versions" => opts.ignore_unknown_versions = true,
        _ => queries.push(arg),
    });

    match resolve(&queries, &opts) {
        Ok(versions) => versions.iter().for_each(|version| println!("{version}")),
        Err(error) => eprintln!("{error}"),
    }
}
