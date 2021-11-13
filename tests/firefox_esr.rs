use browserslist::Opts;
use test_case::test_case;
use util::run_compare;

mod util;

#[test_case("firefox esr"; "firefox")]
#[test_case("Firefox ESR"; "firefox case insensitive")]
#[test_case("ff esr"; "ff")]
#[test_case("FF ESR"; "ff case insensitive")]
#[test_case("fx esr"; "fx")]
#[test_case("Fx ESR"; "fx case insensitive")]
fn firefox_esr(query: &str) {
    run_compare(query, &Opts::new());
}
