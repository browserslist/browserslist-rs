use super::QueryResult;
use crate::opts::Opts;

pub(super) fn browserslist_config(opts: &Opts) -> QueryResult {
    #[cfg(target_arch = "wasm32")]
    {
        crate::resolve(["defaults"], opts)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::execute(opts)
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
        run_compare(query, &Opts::default(), None);
    }
}
