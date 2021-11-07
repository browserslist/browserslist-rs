use browserslist::{opts::Opts, resolve};
use std::env;

fn main() {
    for item in resolve(&env::args().skip(1).collect::<Vec<_>>(), &Opts::default()).unwrap() {
        println!("{}", item);
    }
}
