use browserslist::{Opts, resolve};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn browserslist(query: String, opts: JsValue) -> Result<JsValue, JsValue> {
    let opts: Option<Opts> = serde_wasm_bindgen::from_value(opts)?;

    serde_wasm_bindgen::to_value(
        &resolve([query], &opts.unwrap_or_default())
            .map_err(|e| format!("{}", e))?
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>(),
    )
    .map_err(JsValue::from)
}
