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

pub fn parse(query: &str) -> impl Iterator<Item = Query<'_>> {
    REGEX_OR
        .split(query)
        .map(|s| {
            REGEX_AND.split(s).enumerate().map(|(i, text)| {
                if i == 0 {
                    Query::Or(text)
                } else {
                    Query::And(text)
                }
            })
        })
        .flatten()
}
