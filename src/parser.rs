use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX_OR: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"\s+or\s+|\s*,\s*")
        .case_insensitive(true)
        .build()
        .unwrap()
});

static REGEX_AND: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"\s+and\s+")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub enum Query<'a> {
    And(&'a str),
    Or(&'a str),
}

pub fn parse(query: &str) -> impl Iterator<Item = Query> {
    REGEX_OR.split(query).into_iter().flat_map(|s| {
        REGEX_AND.split(s).enumerate().map(|(i, text)| {
            if i == 0 {
                Query::Or(text)
            } else {
                Query::And(text)
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
