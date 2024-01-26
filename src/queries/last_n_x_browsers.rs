use super::{count_filter_versions, Distrib, QueryResult};
use crate::{data::caniuse::get_browser_stat, error::Error, opts::Opts};

pub(super) fn last_n_x_browsers(count: usize, name: &str, opts: &Opts) -> QueryResult {
    let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
        .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
    let count = count_filter_versions(name, opts.mobile_to_desktop, count);

    let distribs = stat
        .version_list
        .iter()
        .filter(|version| version.release_date.is_some())
        .rev()
        .take(count)
        .map(|version| Distrib::new(name, version.version))
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 ie versions"; "basic")]
    #[test_case("last 2 safari versions"; "do not include unreleased versions")]
    #[test_case("last 1 ie version"; "support pluralization")]
    #[test_case("last 01 Explorer version"; "alias")]
    #[test_case("Last 01 IE Version"; "case insensitive")]
    #[test_case("last 4 android versions"; "android 1")]
    #[test_case("last 5 android versions"; "android 2")]
    #[test_case("last 31 android versions"; "android 3")]
    #[test_case("last 4 op_mob versions"; "op_mob 1")]
    #[test_case("last 5 op_mob versions"; "op_mob 2")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
