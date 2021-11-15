use super::{Distrib, Selector, SelectorResult};
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+electron\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNElectronSelector;

impl Selector for LastNElectronSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if let Some(cap) = REGEX.captures(text) {
            let count: usize = cap[1].parse().map_err(Error::ParseVersionsCount)?;
            let versions = ELECTRON_VERSIONS
                .iter()
                .rev()
                .take(count)
                .map(|(_, version)| Distrib::new("chrome", version))
                .collect();
            Ok(Some(versions))
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

    #[test_case("last 2 electron versions"; "basic")]
    #[test_case("last 2 Electron versions"; "case insensitive")]
    #[test_case("last 2 electron version"; "support pluralization")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
