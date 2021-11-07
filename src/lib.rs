use parser::{parse, Query};
pub use queries::Distrib;
use std::cmp::Ordering;

mod data;
/// error handling
pub mod error;
/// browserslist options
pub mod opts;
mod parser;
mod queries;

fn semver_compare(a: &str, b: &str) -> Ordering {
    a.split('.')
        .zip(b.split('.'))
        .fold(Ordering::Equal, |ord, (a, b)| {
            if ord == Ordering::Equal {
                // this is intentional: version comes from high to low
                b.parse::<i32>()
                    .unwrap_or(0)
                    .cmp(&a.parse::<i32>().unwrap_or(0))
            } else {
                ord
            }
        })
}

pub fn resolve<'a>(
    queries: &'a [impl AsRef<str>],
    opts: &opts::Opts,
) -> Result<Vec<Distrib<'a>>, error::Error> {
    let result = queries
        .iter()
        .map(|query| parse(query.as_ref()))
        .flatten()
        .try_fold(vec![], |mut result, current| {
            let query_string = match current {
                Query::And(s) => s,
                Query::Or(s) => s,
            };

            let is_exclude = query_string.starts_with("not");
            let query_string = if is_exclude {
                &query_string[4..]
            } else {
                query_string
            };

            let mut queries = queries::query(query_string, opts)?;
            if is_exclude {
                result.retain(|q| !queries.contains(q));
            } else {
                match current {
                    Query::And(_) => {
                        result.retain(|q| queries.contains(q));
                    }
                    Query::Or(_) => {
                        result.append(&mut queries);
                    }
                }
            }

            Ok::<_, error::Error>(result)
        });

    let mut result = result?;
    result.sort_by(|a, b| match a.name().cmp(b.name()) {
        Ordering::Equal => {
            let version_a = a.version().split('-').next().unwrap();
            let version_b = b.version().split('-').next().unwrap();
            semver_compare(version_a, version_b)
        }
        ord => ord,
    });
    result.dedup();

    Ok(result)
}
