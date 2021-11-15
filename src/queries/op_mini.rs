use super::{Distrib, Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(?:operamini|op_mini)\s+all$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct OperaMiniSelector;

impl Selector for OperaMiniSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            Ok(Some(vec![Distrib::new("op_mini", "all")]))
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

    #[test_case("op_mini all"; "short")]
    #[test_case("Op_Mini All"; "short case insensitive")]
    #[test_case("operamini all"; "long")]
    #[test_case("OperaMini All"; "long case insensitive")]
    #[test_case("op_mini    all"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
