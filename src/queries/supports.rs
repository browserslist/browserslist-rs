use super::{Distrib, QueryResult};
use crate::{Opts, error::Error, parser::SupportKind};
use browserslist_data::caniuse::{
    VersionDetail, features::get_feature_stat, get_browser_stat, to_desktop_name,
};
use either::Either;

const Y: u8 = 1;
const A: u8 = 2;

pub(super) fn supports(name: &str, kind: Option<SupportKind>, opts: &Opts) -> QueryResult {
    let include_partial = matches!(kind, Some(SupportKind::Partially) | None);

    if let Some(feature) = get_feature_stat(name) {
        let distribs = feature
            .iter()
            .filter_map(|(name, versions)| {
                get_browser_stat(name, opts.mobile_to_desktop)
                    .map(|(name, stat)| (name, stat, versions))
            })
            .flat_map(|(name, browser_stat, versions)| {
                let desktop_name = opts
                    .mobile_to_desktop
                    .then_some(to_desktop_name(name))
                    .flatten();
                let Some(desktop_name) = desktop_name else {
                    return Either::Left(
                        versions
                            .spans(browser_stat.len())
                            .filter(|(_, flags)| is_supported(*flags, include_partial))
                            .flat_map(move |(positions, _)| &browser_stat[positions])
                            .map(move |version| Distrib::new(name, version.version())),
                    );
                };

                let mobile_list = get_browser_stat(name, false).map_or(&[][..], |(_, list)| list);
                let mobile_position = move |version: &VersionDetail| {
                    mobile_list
                        .iter()
                        .position(|mobile| mobile.version() == version.version())
                };
                let check_desktop = browser_stat
                    .iter()
                    .rev()
                    .filter(|version| version.released)
                    .find_map(mobile_position)
                    .is_some_and(|position| {
                        is_supported(versions.flag_at(position), include_partial)
                    });
                let desktop = check_desktop
                    .then(|| {
                        feature
                            .get(desktop_name)
                            .zip(get_browser_stat(desktop_name, false))
                    })
                    .flatten();
                Either::Right(browser_stat.iter().enumerate().filter_map(
                    move |(index, version)| {
                        let flags = match mobile_position(version) {
                            Some(position) => versions.flag_at(position),
                            None => {
                                let (desktop_versions, (_, desktop_list)) = desktop?;
                                let position =
                                    (index + desktop_list.len()).checked_sub(browser_stat.len())?;
                                if desktop_list.get(position)?.version() == version.version() {
                                    desktop_versions.flag_at(position)
                                } else {
                                    None
                                }
                            }
                        };
                        is_supported(flags, include_partial)
                            .then(|| Distrib::new(name, version.version()))
                    },
                ))
            })
            .collect();
        Ok(distribs)
    } else {
        Err(Error::UnknownBrowserFeature(name.to_string()))
    }
}

fn is_supported(flags: u8, include_partial: bool) -> bool {
    flags & Y != 0 || include_partial && flags & A != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        opts::Opts,
        test::{run_compare, should_failed},
    };
    use test_case::test_case;

    #[test_case("supports objectrtc"; "case 1")]
    #[test_case("supports    rtcpeerconnection"; "case 2")]
    #[test_case("supports        arrow-functions"; "case 3")]
    #[test_case("partially supports rtcpeerconnection"; "partially")]
    #[test_case("fully     supports rtcpeerconnection"; "fully")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::default(), None);
    }

    #[test_case("supports filesystem"; "case 1")]
    #[test_case("supports  font-smooth"; "case 2")]
    fn mobile_to_desktop(query: &str) {
        run_compare(
            query,
            &Opts {
                mobile_to_desktop: true,
                ..Default::default()
            },
            None,
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(
            should_failed("supports xxxyyyzzz", &Opts::default()),
            Error::UnknownBrowserFeature(String::from("xxxyyyzzz"))
        );
    }
}
