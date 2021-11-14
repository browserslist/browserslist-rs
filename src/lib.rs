use parser::{parse, Query};
use std::cmp::Ordering;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
pub use {error::Error, opts::Opts, queries::Distrib};

mod data;
mod error;
mod opts;
mod parser;
mod queries;
mod semver;
#[cfg(test)]
mod test;

/// Execute browserslist querying.
pub fn resolve<I, S>(queries: I, opts: &Opts) -> Result<Vec<Distrib>, Error>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
{
    let mut distribs = vec![];

    for query in queries {
        parse(query.as_ref()).try_fold(&mut distribs, |distribs, current| {
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
            version_b
                .parse::<semver::Version>()
                .unwrap_or_default()
                .cmp(&version_a.parse().unwrap_or_default())
        }
        ord => ord,
    });
    distribs.dedup();

    Ok(distribs)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "resolveToStrings")]
pub fn resolve_to_strings(query: String, opts: JsValue) -> Result<JsValue, JsValue> {
    let opts: Option<Opts> = opts.into_serde().unwrap_or_default();

    serde_wasm_bindgen::to_value(
        &resolve([query], &opts.unwrap_or_default())
            .map_err(|e| format!("{}", e))?
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>(),
    )
    .map_err(JsValue::from)
}
