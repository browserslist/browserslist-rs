use once_cell::sync::Lazy;

pub static ELECTRON_VERSIONS: Lazy<Vec<(f32, String)>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/electron-to-chromium.json"
    )))
    .unwrap()
});
