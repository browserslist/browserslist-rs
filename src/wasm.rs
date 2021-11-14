use crate::{opts::Opts, resolve};
use wasm_bindgen::prelude::*;

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
