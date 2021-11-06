use browserslist::resolve;
use std::env;

fn main() {
    let queries = env::args().skip(1).collect::<Vec<_>>();
    let r = resolve(&queries);
    for item in r {
        println!("{}", item);
    }
}
