use browserslist::resolve;
use std::env;

fn main() {
    for item in resolve(&env::args().skip(1).collect::<Vec<_>>()) {
        println!("{}", item);
    }
}
