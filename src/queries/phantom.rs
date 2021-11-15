use super::{Distrib, Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"phantomjs\s+(1\.9|2\.1)")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct PhantomSelector;

impl Selector for PhantomSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            match &cap[1] {
                "1.9" => Ok(Some(vec![Distrib::new("safari", "5")])),
                "2.1" => Ok(Some(vec![Distrib::new("safari", "6")])),
                _ => unreachable!(),
            }
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

    #[test_case("phantomjs 2.1"; "2.1")]
    #[test_case("PhantomJS 2.1"; "2.1 case insensitive")]
    #[test_case("phantomjs 1.9"; "1.9")]
    #[test_case("PhantomJS 1.9"; "1.9 case insensitive")]
    #[test_case("phantomjs    2.1"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
