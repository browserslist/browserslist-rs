use crate::{config, opts::Opts, resolve};
use napi::{bindgen_prelude::*, JsObject, NodeVersion};
use napi_derive::*;
use once_cell::sync::OnceCell;
use serde_json::{from_value, Value};

pub static CURRENT_NODE: OnceCell<NodeVersion> = OnceCell::new();

#[module_exports]
fn init(_exports: JsObject, env: Env) -> Result<()> {
    let _ = CURRENT_NODE.set(env.get_node_version()?);
    Ok(())
}

#[napi]
fn execute(
    query: Either4<String, Vec<String>, Undefined, Null>,
    opts: Option<Value>,
) -> Result<Vec<String>> {
    let opts = match opts {
        Some(opts) => from_value(opts).map_err(|e| Error::from_reason(format!("{}", e)))?,
        None => Opts::default(),
    };

    let queries = match query {
        Either4::A(query) => vec![query],
        Either4::B(queries) => queries,
        Either4::C(_) | Either4::D(_) => {
            config::load(&opts).map_err(|e| Error::from_reason(format!("{}", e)))?
        }
    };

    resolve(&queries, &opts)
        .map_err(|e| Error::from_reason(format!("{}", e)))
        .map(|distribs| distribs.into_iter().map(|d| d.to_string()).collect())
}
