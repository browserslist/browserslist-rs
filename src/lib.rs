use parser::{parse, Query};
pub use queries::Version;
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
) -> Result<Vec<Version<'a>>, error::Error> {
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
    result.sort_by(
        |Version(browser_a, version_a), Version(browser_b, version_b)| {
            if browser_a == browser_b {
                let version_a = version_a.split('-').next().unwrap();
                let version_b = version_b.split('-').next().unwrap();
                semver_compare(version_a, version_b)
            } else {
                browser_a.cmp(browser_b)
            }
        },
    );
    result.dedup();

    Ok(result)
}
