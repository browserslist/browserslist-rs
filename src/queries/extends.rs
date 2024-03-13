use super::QueryResult;
use crate::{error::Error, opts::Opts};

#[cfg(test)]
static BASE_TEST_DIR: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| std::env::temp_dir().join("browserslist-test-pkgs"));

#[cfg(target_arch = "wasm32")]
pub(super) fn extends(pkg: &str, opts: &Opts) -> QueryResult {
    if opts.dangerous_extend {
        Err(Error::UnsupportedExtends)
    } else {
        check_extend_name(pkg).map(|_| Default::default())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn extends(pkg: &str, opts: &Opts) -> QueryResult {
    use crate::{config, resolve};
    use std::{env, process};

    let dangerous_extend =
        opts.dangerous_extend || env::var("BROWSERSLIST_DANGEROUS_EXTEND").is_ok();
    if !dangerous_extend {
        check_extend_name(pkg)?;
    }

    let mut command = process::Command::new("node");
    command.args(["-p", &format!("JSON.stringify(require('{pkg}'))")]);
    #[cfg(test)]
    command.current_dir(&*BASE_TEST_DIR);
    let output = command
        .output()
        .map_err(|_| Error::UnsupportedExtends)?
        .stdout;
    let config = serde_json::from_str(&String::from_utf8_lossy(&output))
        .map_err(|_| Error::FailedToResolveExtend(pkg.to_string()))?;

    resolve(config::load_with_config(config, opts)?, opts)
}

fn check_extend_name(pkg: &str) -> Result<(), Error> {
    let unscoped = pkg
        .strip_prefix('@')
        .and_then(|s| s.find('/').and_then(|i| s.get(i + 1..)))
        .unwrap_or(pkg);
    if !unscoped.starts_with("browserslist-config-")
        && !(pkg.starts_with('@') && unscoped == "browserslist-config")
    {
        return Err(Error::InvalidExtendName(
            "Browserslist config needs `browserslist-config-` prefix.",
        ));
    }
    if unscoped.contains('.') {
        return Err(Error::InvalidExtendName(
            "`.` not allowed in Browserslist config name.",
        ));
    }
    if pkg.contains("node_modules") {
        return Err(Error::InvalidExtendName(
            "`node_modules` not allowed in Browserslist config.",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        opts::Opts,
        test::{run_compare, should_failed},
    };
    use serde_json::json;
    use std::fs;
    use test_case::test_case;

    fn mock(name: &str, value: serde_json::Value) {
        let dir = BASE_TEST_DIR.join("node_modules").join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("index.js"),
            format!(
                "module.exports = {}",
                serde_json::to_string(&value).unwrap()
            ),
        )
        .unwrap();
    }

    fn clean(name: &str) {
        let _ = fs::remove_dir_all(BASE_TEST_DIR.join("node_modules").join(name));
    }

    #[test_case("browserslist-config-test", json!(["ie 11"]), "extends browserslist-config-test"; "package")]
    #[test_case("browserslist-config-test-file/ie", json!(["ie 11"]), "extends browserslist-config-test-file/ie"; "file in package")]
    #[test_case("@scope/browserslist-config-test", json!(["ie 11"]), "extends @scope/browserslist-config-test"; "scoped package")]
    #[test_case("@example.com/browserslist-config-test", json!(["ie 11"]), "extends @example.com/browserslist-config-test"; "scoped package with dot in name")]
    #[test_case("@scope/browserslist-config-test-file/ie", json!(["ie 11"]), "extends @scope/browserslist-config-test-file/ie"; "file in scoped package")]
    #[test_case("@scope/browserslist-config", json!(["ie 11"]), "extends @scope/browserslist-config"; "file-less scoped package")]
    #[test_case("browserslist-config-rel", json!(["ie 9-10"]), "extends browserslist-config-rel and not ie 9"; "with override")]
    #[test_case("browserslist-config-with-env-a", json!({ "someEnv": ["ie 10"] }), "extends browserslist-config-with-env-a"; "no default env")]
    #[test_case("browserslist-config-with-defaults", json!({ "defaults": ["ie 10"] }), "extends browserslist-config-with-defaults"; "default env")]
    fn valid(pkg: &str, value: serde_json::Value, query: &str) {
        mock(pkg, value);
        run_compare(query, &Default::default(), Some(&BASE_TEST_DIR));
        clean(pkg);
    }

    #[test]
    fn dangerous_extend() {
        mock("pkg", json!(["ie 11"]));
        run_compare(
            "extends pkg",
            &Opts {
                dangerous_extend: true,
                ..Default::default()
            },
            Some(&BASE_TEST_DIR),
        );
        clean("pkg");
    }

    #[test]
    fn recursively_import() {
        mock(
            "browserslist-config-a",
            json!(["extends browserslist-config-b", "ie 9"]),
        );
        mock("browserslist-config-b", json!(["ie 10"]));
        run_compare(
            "extends browserslist-config-a",
            &Default::default(),
            Some(&BASE_TEST_DIR),
        );
        clean("browserslist-config-a");
        clean("browserslist-config-b");
    }

    #[test]
    fn specific_env() {
        mock("browserslist-config-with-env-b", json!(["ie 11"]));
        run_compare(
            "extends browserslist-config-with-env-b",
            &Opts {
                env: Some("someEnv".into()),
                ..Default::default()
            },
            Some(&BASE_TEST_DIR),
        );
        clean("pkg");
    }

    #[test_case("browserslist-config-wrong", json!(null), "extends browserslist-config-wrong"; "empty export")]
    fn invalid(pkg: &str, value: serde_json::Value, query: &str) {
        mock(pkg, value);
        assert!(matches!(
            should_failed(query, &Default::default()),
            Error::FailedToResolveExtend(..)
        ));
        clean(pkg);
    }

    #[test_case("extends thing-without-prefix"; "without prefix")]
    #[test_case("extends browserslist-config-package/../something"; "has dot")]
    #[test_case("extends browserslist-config-test/node_modules/a"; "has node_modules")]
    fn invalid_name(query: &str) {
        assert!(matches!(
            should_failed(query, &Default::default()),
            Error::InvalidExtendName(..)
        ));
    }
}
