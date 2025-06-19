use super::{Distrib, QueryResult};
use crate::data::node;

pub(super) fn last_n_node(count: usize) -> QueryResult {
    let distribs = node::versions()
        .iter()
        .rev()
        .take(count)
        .map(|version| Distrib::new("node", *version))
        .collect();
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("last 2 node versions"; "basic")]
    #[test_case("last 2 Node versions"; "case insensitive")]
    #[test_case("last 2 node version"; "support pluralization")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
