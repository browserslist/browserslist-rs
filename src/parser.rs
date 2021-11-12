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

pub fn parse<'a>(query: &'a str) -> impl Iterator<Item = Query<'a>> {
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
