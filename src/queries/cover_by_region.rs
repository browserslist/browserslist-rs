use super::{Distrib, QueryResult};
use crate::{data::caniuse::region::get_usage_by_region, error::Error};
use std::ops::ControlFlow;

pub(super) fn cover_by_region(coverage: f32, region: &str) -> QueryResult {
    let normalized_region = if region.len() == 2 {
        region.to_uppercase()
    } else {
        region.to_lowercase()
    };

    if let Some(region_data) = get_usage_by_region(&normalized_region) {
        let result = region_data.iter().try_fold(
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
    } else {
        Err(Error::UnknownRegion(region.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case("cover 0.1% in US"; "country")]
    #[test_case("Cover 0.1% in us"; "country case insensitive")]
    #[test_case("cover 0.1% in alt-eu"; "country alt")]
    #[test_case("Cover 0.1% in Alt-EU"; "country alt case insensitive")]
    fn valid(query: &str) {
        run_compare(query, &Opts::new(), None);
    }
}
