use parser::{parse, Query};
use std::cmp::Ordering;
pub use {error::Error, opts::Opts, queries::Distrib};

mod data;
mod error;
mod opts;
mod parser;
mod queries;
mod semver;

pub fn resolve<I, S>(queries: I, opts: &Opts) -> Result<Vec<Distrib>, Error>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
{
    let mut distribs = vec![];

    for query in queries {
        let query = query.as_ref();
        parse(&query).try_fold(&mut distribs, |distribs, current| {
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
                distribs.retain(|q| !queries.contains(q));
            } else {
                match current {
                    Query::And(_) => {
                        distribs.retain(|q| queries.contains(q));
                    }
                    Query::Or(_) => {
                        distribs.append(&mut queries);
                    }
                }
            }

            Ok::<_, Error>(distribs)
        })?;
    }

    distribs.sort_by(|a, b| match a.name().cmp(b.name()) {
        Ordering::Equal => {
            let version_a = a.version().split('-').next().unwrap();
            let version_b = b.version().split('-').next().unwrap();
            semver::reverse_compare(version_a, version_b)
        }
        ord => ord,
    });
    distribs.dedup();

    Ok(distribs)
}
