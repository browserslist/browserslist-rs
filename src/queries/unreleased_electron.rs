use super::{Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^unreleased\s+electron\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct UnreleasedElectronSelector;

impl Selector for UnreleasedElectronSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            Ok(Some(vec![]))
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

    #[test_case("unreleased electron versions"; "basic")]
    #[test_case("Unreleased Electron Versions"; "case insensitive")]
    #[test_case("unreleased electron version"; "support pluralization")]
    #[test_case("unreleased   electron      versions"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
