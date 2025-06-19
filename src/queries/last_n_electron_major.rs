use super::{Distrib, QueryResult};
use crate::data::electron;
use itertools::Itertools;

pub(super) fn last_n_electron_major(count: usize) -> QueryResult {
    let minimum = electron::versions()
        .rev()
        .dedup()
        .nth(count - 1)
        .map(|(electron_version, _)| electron_version)
        .unwrap_or(0.0);

    let distribs = electron::versions()
        .filter(|(electron_version, _)| *electron_version >= minimum)
        .rev()
        .map(|(_, chromium_version)| Distrib::new("chrome", chromium_version))
        .collect();

    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("last 2 electron major versions"; "basic")]
    #[test_case("last 2 Electron major versions"; "case insensitive")]
    #[test_case("last 2 electron major version"; "support pluralization")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
