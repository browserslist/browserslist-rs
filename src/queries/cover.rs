use super::{Distrib, QueryResult};
use crate::data::caniuse::CANIUSE_GLOBAL_USAGE;
use std::ops::ControlFlow;

pub(super) fn cover(coverage: f32) -> QueryResult {
    let result = CANIUSE_GLOBAL_USAGE.iter().try_fold(
        (vec![], 0.0),
        |(mut distribs, total), (name, version, usage)| {
            if total >= coverage || *usage == 0.0 {
                ControlFlow::Break((distribs, total))
            } else {
                distribs.push(Distrib::new(name, *version));
                ControlFlow::Continue((distribs, total + usage))
            }
        },
    );
    match result {
        ControlFlow::Break((distribs, _)) => Ok(distribs),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("cover 0.1%"; "global")]
    #[test_case("Cover 0.1%"; "global case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new(), None);
    }
}
