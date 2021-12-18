use super::{Distrib, QueryResult};
use crate::{
    data::caniuse::{get_browser_stat, BROWSER_VERSION_ALIASES},
    error::Error,
    opts::Opts,
    parser::Comparator,
    semver::Version,
};
use ustr::Ustr;

pub(super) fn browser_unbounded_range(
    name: &str,
    comparator: Comparator,
    version: &str,
    opts: &Opts,
) -> QueryResult {
    let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
        .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
    let version: Version = BROWSER_VERSION_ALIASES
        .get(&Ustr::from(name))
        .and_then(|alias| alias.get(version).copied())
        .unwrap_or(version)
        .parse()
        .unwrap_or_default();

    let distribs = stat
        .version_list
        .iter()
        .filter(|version| version.release_date.is_some())
        .map(|version| &*version.version)
        .filter(|v| {
            let v: Version = v.parse().unwrap_or_default();
            match comparator {
                Comparator::Greater => v > version,
                Comparator::Less => v < version,
                Comparator::GreaterOrEqual => v >= version,
                Comparator::LessOrEqual => v <= version,
            }
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

    #[test_case("ie > 9"; "greater")]
    #[test_case("ie >= 10"; "greater or equal")]
    #[test_case("ie < 10"; "less")]
    #[test_case("ie <= 9"; "less or equal")]
    #[test_case("Explorer > 10"; "case insensitive")]
    #[test_case("android >= 4.2"; "android 1")]
    #[test_case("android >= 4.3"; "android 2")]
    #[test_case("ie<=9"; "no spaces")]
    #[test_case("and_qq > 0"; "browser with one version")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test_case("chromeandroid >= 52 and chromeandroid < 54"; "chrome")]
    fn mobile_to_desktop(query: &str) {
        run_compare(query, Opts::new().mobile_to_desktop(true));
    }

    #[test_case(
        "unknow > 10", Error::BrowserNotFound(String::from("unknow"));
        "unknown browser"
    )]
    fn invalid(query: &str, error: Error) {
        assert_eq!(should_failed(query, &Opts::new()), error);
    }
}
