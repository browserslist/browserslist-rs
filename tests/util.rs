use browserslist::{resolve, Opts};
use std::process::Command;

pub fn run_compare(query: &str, opts: &Opts) {
    let output = String::from_utf8(
        Command::new("./node_modules/.bin/browserslist")
            .arg(query)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let expected = output.trim().split('\n').collect::<Vec<_>>();

    let actual = resolve([query], opts)
        .unwrap()
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>();

    assert_eq!(expected, actual);
}
