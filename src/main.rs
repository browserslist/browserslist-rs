use browserslist::resolve;
use std::env;

fn main() {
    let r = resolve(env::args().nth(1).unwrap().as_str());
    dbg!(r);
}
