use super::{Distrib, QueryResult};
use crate::data::electron::ELECTRON_VERSIONS;

pub(super) fn last_n_electron(count: usize) -> QueryResult {
    let distribs = ELECTRON_VERSIONS
        .iter()
        .rev()
        .take(count)
        .map(|(_, version)| Distrib::new("chrome", *version))
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("last 2 electron versions"; "basic")]
    #[test_case("last 2 Electron versions"; "case insensitive")]
    #[test_case("last 2 electron version"; "support pluralization")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new(), None);
    }
}
