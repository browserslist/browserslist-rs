use super::{Distrib, Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(?:firefox|ff|fx)\s+esr$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct FirefoxESRSelector;

impl Selector for FirefoxESRSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            Ok(Some(vec![
                Distrib::new("firefox", "78"),
                Distrib::new("firefox", "91"),
            ]))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("firefox esr"; "firefox")]
    #[test_case("Firefox ESR"; "firefox case insensitive")]
    #[test_case("ff esr"; "ff")]
    #[test_case("FF ESR"; "ff case insensitive")]
    #[test_case("fx esr"; "fx")]
    #[test_case("Fx ESR"; "fx case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
