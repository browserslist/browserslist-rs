use browserslist::resolve;
use std::env;

fn main() {
    let queries = env::args().skip(1).collect::<Vec<_>>();
    let mut r = resolve(&queries);
    r.sort();
    for item in r {
        println!("{}", item);
    }
}
