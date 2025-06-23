use super::{Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, normalize_version},
    error::Error,
    opts::Opts,
    semver::Version,
};

pub(super) fn browser_bounded_range(name: &str, from: &str, to: &str, opts: &Opts) -> QueryResult {
    let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
        .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
    let from: Version = normalize_version(name, stat, from)
        .unwrap_or(from)
        .parse()
        .unwrap_or_default();
    let to: Version = normalize_version(name, stat, to)
        .unwrap_or(to)
        .parse()
        .unwrap_or_default();

    let distribs = stat
        .iter()
        .filter(|version| version.released)
        .map(|version| version.version())
        .filter(|version| {
            let version = version.parse().unwrap_or_default();
            from <= version && version <= to
        })
        .map(|version| Distrib::new(name, version))
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{run_compare, should_failed};
    use test_case::test_case;

    #[test_case("ie 8-10"; "basic")]
    #[test_case("ie 8   -  10"; "more spaces")]
    #[test_case("ie 1-12"; "out of range")]
    #[test_case("android 4.3-37"; "android")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test_case("and_chr 52-53"; "chrome")]
    #[test_case("android 4.4-38"; "android")]
    fn mobile_to_desktop(query: &str) {
        run_compare(
            query,
            &Opts {
                mobile_to_desktop: true,
                ..Default::default()
            },
            None,
        );
    }

    #[test_case(
        "unknown 4-7", Error::BrowserNotFound(String::from("unknown"));
        "unknown browser"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::default()), error);
    }
}
