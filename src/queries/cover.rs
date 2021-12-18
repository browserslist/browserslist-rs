use super::{Distrib, QueryResult};
use crate::data::caniuse::CANIUSE_GLOBAL_USAGE;
use std::ops::ControlFlow;

pub(super) fn cover(coverage: f32) -> QueryResult {
    let result = CANIUSE_GLOBAL_USAGE.iter().try_fold(
        (vec![], 0.0),
        |(mut versions, total), (name, version, usage)| {
            if total >= coverage || *usage == 0.0 {
                ControlFlow::Break((versions, total))
            } else {
                versions.push(Distrib::new(name, version));
                ControlFlow::Continue((versions, total + usage))
            }
        },
    );
    let distribs = match result {
        ControlFlow::Break((versions, _)) => versions,
        _ => unreachable!(),
    };
    Ok(distribs)
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("cover 0.1%"; "global")]
    #[test_case("Cover 0.1%"; "global case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new());
    }
}
