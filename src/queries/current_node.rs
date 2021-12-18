use super::{Distrib, QueryResult};
use crate::error::Error;

pub(super) fn current_node() -> QueryResult {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::{global, Reflect};

        let obj_process = Reflect::get(&global(), &"process".into())
            .map_err(|_| Error::UnsupportedCurrentNode)?;
        let obj_versions = Reflect::get(&obj_process, &"versions".into())
            .map_err(|_| Error::UnsupportedCurrentNode)?;
        let version = Reflect::get(&obj_versions, &"node".into())
            .map_err(|_| Error::UnsupportedCurrentNode)?
            .as_string()
            .ok_or(Error::UnsupportedCurrentNode)?;
        Ok(vec![Distrib::new("node", version)])
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "node"))]
    {
        use crate::node::CURRENT_NODE;

        let version = CURRENT_NODE.get().ok_or(Error::UnsupportedCurrentNode)?;
        Ok(vec![Distrib::new(
            "node",
            format!("{}.{}.{}", version.major, version.minor, version.patch),
        )])
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "node")))]
    {
        use std::process::Command;

        let output = Command::new("node")
            .arg("-v")
            .output()
            .map_err(|_| Error::UnsupportedCurrentNode)?;
        let version = String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_start_matches('v')
            .to_owned();

        Ok(vec![Distrib::new("node", version)])
    }
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("current node"; "basic")]
    #[test_case("Current Node"; "case insensitive")]
    #[test_case("current      node"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
