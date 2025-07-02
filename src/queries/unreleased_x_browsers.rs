use super::{Distrib, QueryResult};
use crate::{error::Error, opts::Opts};
use browserslist_data::caniuse::get_browser_stat;

pub(super) fn unreleased_x_browsers(name: &str, opts: &Opts) -> QueryResult {
    let (name, version_list) = get_browser_stat(name, opts.mobile_to_desktop)
        .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
    let distribs = version_list
        .iter()
        .filter(|version| !version.released)
        .map(|version| Distrib::new(name, version.version()))
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("unreleased edge versions"; "basic")]
    #[test_case("Unreleased Chrome Versions"; "case insensitive")]
    #[test_case("unreleased firefox version"; "support pluralization")]
    #[test_case("unreleased    safari     versions"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
