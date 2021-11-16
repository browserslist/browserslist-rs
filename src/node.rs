use crate::{opts::Opts, resolve};
use napi::bindgen_prelude::*;
use napi_derive::*;

#[napi]
fn execute(query: Either<String, Vec<String>>, opts: Option<Opts>) -> Result<Vec<String>> {
    let queries = match query {
        Either::A(query) => vec![query],
        Either::B(queries) => queries,
    };
    let opts = opts.unwrap_or_default();

    resolve(&queries, &opts)
        .map_err(|e| Error::from_reason(format!("{}", e)))
        .map(|distribs| distribs.into_iter().map(|d| d.to_string()).collect())
}
