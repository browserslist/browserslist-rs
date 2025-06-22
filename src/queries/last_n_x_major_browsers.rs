use super::{count_filter_versions, Distrib, QueryResult};
use crate::{data::caniuse::get_browser_stat, error::Error, opts::Opts};
use itertools::Itertools;

pub(super) fn last_n_x_major_browsers(count: usize, name: &str, opts: &Opts) -> QueryResult {
    let (name, version_list) = get_browser_stat(name, opts.mobile_to_desktop)
        .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
    let count = count_filter_versions(name, opts.mobile_to_desktop, count);
    let minimum = version_list
        .iter()
        .filter(|version| version.released)
        .map(|version| version.version.as_str())
        .rev()
        .map(|version| version.split('.').next().unwrap())
        .dedup()
        .nth(count - 1)
        .and_then(|minimum| minimum.parse().ok())
        .unwrap_or(0);

    let distribs = version_list
        .iter()
        .filter(|version| version.released)
        .map(|version| version.version.as_str())
        .filter(move |version| version.split('.').next().unwrap().parse().unwrap_or(0) >= minimum)
        .rev()
        .map(move |version| Distrib::new(name, version))
        .collect();

    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 edge major versions"; "basic")]
    #[test_case("last 1 bb major version"; "support pluralization")]
    #[test_case("last 3 Chrome major versions"; "case insensitive")]
    #[test_case("last 2 android major versions"; "android")]
    #[test_case("last 2 bb major versions"; "non-sequential version numbers")]
    #[test_case("last 3 bb major versions"; "more versions than have been released")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test]
    fn mobile_to_desktop() {
        run_compare(
            "last 2 android major versions",
            &Opts {
                mobile_to_desktop: true,
                ..Default::default()
            },
            None,
        );
    }
}
