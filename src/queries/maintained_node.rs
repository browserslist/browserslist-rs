use super::{Distrib, QueryResult};
use crate::data::node::{NODE_VERSIONS, RELEASE_SCHEDULE};
use chrono::Local;

pub(super) fn maintained_node() -> QueryResult {
    let now = Local::now().naive_local();

    let versions = RELEASE_SCHEDULE
        .iter()
        .filter(|(_, (start, end))| *start < now && now < *end)
        .filter_map(|(version, _)| {
            NODE_VERSIONS
                .iter()
                .rev()
                .find(|v| v.split('.').next().unwrap() == version)
        })
        .map(|version| Distrib::new("node", version))
        .collect();
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("maintained node versions"; "basic")]
    #[test_case("Maintained Node Versions"; "case insensitive")]
    #[test_case("maintained   node     versions"; "more spaces")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
