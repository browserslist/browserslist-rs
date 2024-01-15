use browserslist::{resolve, Opts};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    mobile_to_desktop: bool,

    #[arg(long)]
    ignore_unknown_versions: bool,

    queries: Vec<String>,
}

fn main() {
    let args = Args::parse();

    match resolve(
        &args.queries,
        Opts::new()
            .mobile_to_desktop(args.mobile_to_desktop)
            .ignore_unknown_versions(args.ignore_unknown_versions),
    ) {
        Ok(versions) => {
            for version in versions {
                println!("{}", version)
            }
        }
        Err(error) => eprintln!("{}", error),
    };
}
