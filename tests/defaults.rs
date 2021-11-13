use browserslist::Opts;
use test_case::test_case;
use util::run_compare;

mod util;

#[test_case("defaults", &Opts::new(); "no options")]
#[test_case("Defaults", &Opts::new(); "case insensitive")]
#[test_case("defaults", &Opts::new().mobile_to_desktop(true); "respect options")]
fn defaults(query: &str, opts: &Opts) {
    run_compare(query, &opts);
}
