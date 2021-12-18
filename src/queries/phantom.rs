use super::{Distrib, QueryResult};

pub(super) fn phantom(is_later_version: bool) -> QueryResult {
    let version = if is_later_version { "6" } else { "5" };
    Ok(vec![Distrib::new("safari", version)])
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("phantomjs 2.1"; "2.1")]
    #[test_case("PhantomJS 2.1"; "2.1 case insensitive")]
    #[test_case("phantomjs 1.9"; "1.9")]
    #[test_case("PhantomJS 1.9"; "1.9 case insensitive")]
    #[test_case("phantomjs    2.1"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
