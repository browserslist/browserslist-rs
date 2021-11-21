use crate::{resolve, Error, Opts};
use std::process::Command;

pub fn run_compare(query: &str, opts: &Opts) {
    let mut command = Command::new("./node_modules/.bin/browserslist");
    if opts.mobile_to_desktop {
        command.arg("--mobile-to-desktop");
    }
    if opts.ignore_unknown_versions {
        command.arg("--ignore-unknown-versions");
    }
    command.arg(query);
    let output = String::from_utf8(command.output().unwrap().stdout).unwrap();
    let expected = output
        .trim()
        .split('\n')
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let actual = resolve([query], opts)
        .unwrap()
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>();

    assert_eq!(expected, actual);
}

pub fn should_failed(query: &str, opts: &Opts) -> Error {
    resolve([query], opts).unwrap_err()
}
