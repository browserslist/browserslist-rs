use once_cell::sync::Lazy;

pub static ELECTRON_VERSIONS: Lazy<Vec<(f32, String)>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../data/electron-to-chromium.json")).unwrap()
});
