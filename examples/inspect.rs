use browserslist::{resolve, Opts};
use clap::{App, Arg};

fn main() {
    let matches = App::new("Browserslist")
        .arg(
            Arg::with_name("mobile_to_desktop")
                .long("mobile-to-desktop")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("ignore_unknown_versions")
                .long("ignore-unknown-versions")
                .takes_value(false),
        )
        .arg(Arg::with_name("queries"))
        .get_matches();

    match resolve(
        &vec![matches.value_of("queries").unwrap_or_default()],
        Opts::new()
            .mobile_to_desktop(matches.is_present("mobile_to_desktop"))
            .ignore_unknown_versions(matches.is_present("ignore_unknown_versions")),
    ) {
        Ok(versions) => {
            for version in versions {
                println!("{}", version)
            }
        }
        Err(error) => eprintln!("{}", error),
    };
}
