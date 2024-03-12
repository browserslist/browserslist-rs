use super::QueryResult;
use crate::{opts::Opts, resolve};

pub(super) fn dead(opts: &Opts) -> QueryResult {
    resolve(
        [
            "Baidu >= 0",
            "ie <= 11",
            "ie_mob <= 11",
            "bb <= 10",
            "op_mob <= 12.1",
            "samsung 4",
        ],
        opts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::Error,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("dead"; "basic")]
    #[test_case("Dead"; "case insensitive")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new(), None);
    }

    #[test_case("> 0%, dead"; "all browsers")]
    fn mobile_to_desktop(query: &str) {
        run_compare(query, Opts::new().mobile_to_desktop(true), None);
    }

    #[test]
    fn invalid() {
        assert_eq!(
            should_failed("not dead", &Opts::new()),
            Error::NotAtFirst(String::from("not dead"))
        );
    }
}
