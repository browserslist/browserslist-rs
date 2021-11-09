use once_cell::sync::Lazy;

pub static NODE_VERSIONS: Lazy<Vec<String>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/node-versions.json"
    )))
    .unwrap()
});
