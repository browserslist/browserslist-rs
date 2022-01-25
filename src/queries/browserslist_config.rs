use super::QueryResult;
use crate::opts::Opts;

pub(super) fn browserslist_config(opts: &Opts) -> QueryResult {
    #[cfg(target = "wasm32-unknown-unknown")]
    {
        crate::resolve(["defaults"], opts)
    }

    #[cfg(not(target = "wasm32-unknown-unknown"))]
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
        run_compare(query, &Opts::new());
    }
}
