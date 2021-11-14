use super::{Distrib, Selector, SelectorResult};
use crate::opts::Opts;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^current\s+node$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct CurrentNodeSelector;

impl Selector for CurrentNodeSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        if REGEX.is_match(text) {
            #[cfg(target_arch = "wasm32")]
            {
                use crate::error::Error;
                use js_sys::{global, Reflect};

                let obj_process = Reflect::get(&global(), &"process".into())
                    .map_err(|_| Error::UnsupportedCurrentNode)?;
                let obj_versions = Reflect::get(&obj_process, &"versions".into())
                    .map_err(|_| Error::UnsupportedCurrentNode)?;
                let version = Reflect::get(&obj_versions, &"node".into())
                    .map_err(|_| Error::UnsupportedCurrentNode)?
                    .as_string()
                    .ok_or(Error::UnsupportedCurrentNode)?;
                Ok(Some(vec![Distrib::new("node", version)]))
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                use std::process::Command;

                let output = Command::new("node").arg("-v").output().unwrap();
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .trim_start_matches('v')
                    .to_owned();

                Ok(Some(vec![Distrib::new("node", version)]))
            }
        } else {
            Ok(None)
        }
    }
}
