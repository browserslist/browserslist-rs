use super::{Selector, SelectorResult};
use crate::{execute, opts::Opts};

pub(super) struct BrowserslistConfigSelector;

impl Selector for BrowserslistConfigSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if text.eq_ignore_ascii_case("browserslist config") {
            execute(opts).map(Some)
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

    #[test_case("browserslist config"; "basic")]
    #[test_case("Browserslist Config"; "case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
