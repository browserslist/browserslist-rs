use browserslist::{resolve, Distrib, Error, Opts};
use test_case::test_case;

mod util;

#[test_case("ie 10", &Opts::new(); "by name")]
#[test_case("IE 10", &Opts::new(); "case insensitive")]
#[test_case("Explorer 10", &Opts::new(); "alias")]
#[test_case("ios 7.0", &Opts::new(); "work with joined versions 1")]
#[test_case("ios 7.1", &Opts::new(); "work with joined versions 2")]
#[test_case("ios 7", &Opts::new(); "allow missing zero 1")]
#[test_case("ios 8.0", &Opts::new(); "allow missing zero 2")]
#[test_case("safari tp", &Opts::new(); "safari tp")]
#[test_case("Safari TP", &Opts::new(); "safari tp case insensitive")]
#[test_case("and_uc 10", &Opts::new(); "cutted version")]
fn browser_accurate(query: &str, opts: &Opts) {
    util::run_compare(query, &opts);
}

#[test_case("unknown 10", Error::BrowserNotFound(String::from("unknown")); "unknown browser")]
#[test_case(
    "IE 1", Error::UnknownBrowserVersion(String::from("IE"), String::from("1"));
    "unknown version"
)]
#[test_case(
    "chrome 70, ie 11, safari 12.2, safari 12",
    Error::UnknownBrowserVersion(String::from("safari"), String::from("12.2"));
    "use correct browser name in error"
)]
fn browser_accurate_invalid(query: &str, error: Error) {
    assert_eq!(util::should_failed(query, &Opts::new()), error);
}

#[test]
fn browser_accurate_ignore_unknown_versions() {
    assert_eq!(
        resolve(["IE 1, IE 9"], &Opts::new().ignore_unknown_versions(true)).unwrap()[0],
        Distrib::new("ie", "9")
    );
}
