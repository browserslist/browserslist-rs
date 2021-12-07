pub enum Query<'a> {
    And(&'a str),
    Or(&'a str),
}

pub fn parse(query: &str) -> impl Iterator<Item = Query> {
    query
        .split(" or ")
        .flat_map(|s| s.split(','))
        .flat_map(|s| {
            s.split(" and ").enumerate().map(|(i, text)| {
                if i == 0 {
                    Query::Or(text.trim())
                } else {
                    Query::And(text.trim())
                }
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("ie >= 6, ie <= 7"; "comma")]
    #[test_case("ie >= 6 and ie <= 7"; "and")]
    #[test_case("ie < 11 and not ie 7"; "and with not")]
    #[test_case("last 1 Baidu version and not <2%"; "with not and one-version browsers as and query")]
    #[test_case("ie >= 6 or ie <= 7"; "or")]
    #[test_case("ie < 11 or not ie 7"; "or with not")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
