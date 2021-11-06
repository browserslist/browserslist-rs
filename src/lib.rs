use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

mod queries;

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

#[derive(Debug)]
enum Query<'a> {
    And(&'a str),
    Or(&'a str),
}

fn parse(query: &str) -> impl Iterator<Item = Query<'_>> {
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

pub fn resolve(queries: &[impl AsRef<str>]) -> Vec<String> {
    queries
        .iter()
        .map(|query| parse(query.as_ref()))
        .flatten()
        .fold(vec![], |mut result, current| {
            match current {
                Query::And(query_string) => {
                    let is_exclude = query_string.starts_with("not");
                    let query_string = if is_exclude {
                        &query_string[4..]
                    } else {
                        query_string
                    };
                    if let Some(queries) = queries::query(query_string) {
                        if is_exclude {
                            result.retain(|q| !queries.contains(q));
                        } else {
                            result.retain(|q| queries.contains(q));
                        }
                    }
                }
                Query::Or(query_string) => {
                    let is_exclude = query_string.starts_with("not");
                    let query_string = if is_exclude {
                        &query_string[4..]
                    } else {
                        query_string
                    };
                    if let Some(mut queries) = queries::query(query_string) {
                        if is_exclude {
                            result.retain(|q| !queries.contains(q));
                        } else {
                            result.append(&mut queries);
                        }
                    }
                }
            }
            result
        })
}
