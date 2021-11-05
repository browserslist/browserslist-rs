use browserslist::resolve;
use std::env;

fn main() {
    let mut r = resolve(env::args().nth(1).unwrap().as_str());
    r.sort();
    for item in r {
        println!("{}", item);
    }
}
