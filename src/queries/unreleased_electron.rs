use super::QueryResult;

pub(super) fn unreleased_electron() -> QueryResult {
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("unreleased electron versions"; "basic")]
    #[test_case("Unreleased Electron Versions"; "case insensitive")]
    #[test_case("unreleased electron version"; "support pluralization")]
    #[test_case("unreleased   electron      versions"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
