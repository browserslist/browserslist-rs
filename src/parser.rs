use winnow::{
    Result,
    ascii::{Caseless, dec_int, dec_uint, digit1, float, space0, space1},
    combinator::{
        alt, delimited, opt, peek, preceded, repeat, repeat_till, separated_pair, terminated,
    },
    error::ContextError,
    prelude::*,
    token::{any, one_of, take_while},
};

#[derive(Debug, Clone)]
pub enum BaselineKind<'a> {
    WidelyAvailable,
    WidelyAvailableOnDate(&'a str), // "YYYY-MM-DD"
    NewlyAvailable,
    Year(u16),
}

#[derive(Debug, Clone)]
pub enum QueryAtom<'a> {
    Last {
        count: u16,
        major: bool,
        name: Option<&'a str>,
    },
    Unreleased(Option<&'a str>),
    Years(f64),
    Since {
        year: i32,
        month: u32,
        day: u32,
    },
    Percentage {
        comparator: Comparator,
        popularity: f32,
        stats: Stats<'a>,
    },
    Cover {
        coverage: f32,
        stats: Stats<'a>,
    },
    Supports(&'a str, Option<SupportKind>),
    Electron(VersionRange<'a>),
    Node(VersionRange<'a>),
    Browser(&'a str, VersionRange<'a>),
    Baseline {
        kind: BaselineKind<'a>,
        downstream: bool,
        kaios: bool,
    },
    FirefoxESR,
    OperaMini,
    CurrentNode,
    MaintainedNode,
    Phantom(bool),
    BrowserslistConfig,
    Defaults,
    Dead,
    Extends(&'a str),
    Unknown(&'a str), // unnecessary, but for better error report
}

#[derive(Debug, Clone)]
pub enum Stats<'a> {
    Global,
    Region(&'a str),
}

#[derive(Debug, Clone)]
pub enum SupportKind {
    Fully,
    Partially,
}

fn parse_version_keyword<'a>(input: &mut &'a str) -> Result<&'a str> {
    terminated(Caseless("version"), opt('s')).parse_next(input)
}

fn parse_last<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        terminated(Caseless("last"), space1),
        terminated(digit1.verify_map(|s: &str| s.parse().ok()), space1),
        opt(terminated(
            take_while(1.., |c: char| c.is_ascii_alphabetic() || c == '_').verify(|s: &str| {
                !s.eq_ignore_ascii_case("version")
                    && !s.eq_ignore_ascii_case("versions")
                    && !s.eq_ignore_ascii_case("major")
            }),
            space1,
        )),
        opt(terminated(Caseless("major"), space1)),
        parse_version_keyword,
    )
        .map(|(_, count, name, major, _)| {
            if matches!(name, Some(name) if name.eq_ignore_ascii_case("major")) && major.is_none() {
                QueryAtom::Last {
                    count,
                    major: true,
                    name: None,
                }
            } else {
                QueryAtom::Last {
                    count,
                    major: major.is_some(),
                    name,
                }
            }
        })
        .parse_next(input)
}

fn parse_unreleased<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    delimited(
        terminated(Caseless("unreleased"), space1),
        opt(terminated(
            take_while(1.., |c: char| c.is_ascii_alphabetic() || c == '_'),
            space1,
        )),
        parse_version_keyword,
    )
    .map(QueryAtom::Unreleased)
    .parse_next(input)
}

fn parse_years<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    delimited(
        terminated(Caseless("last"), space1),
        terminated(float, space1),
        terminated(Caseless("year"), opt('s')),
    )
    .map(QueryAtom::Years)
    .parse_next(input)
}

fn parse_since<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        terminated(Caseless("since"), one_of([' ', '\t'])),
        dec_int,
        opt(preceded('-', digit1.verify_map(|s: &str| s.parse().ok()))),
        opt(preceded('-', digit1.verify_map(|s: &str| s.parse().ok()))),
    )
        .map(|(_, year, month, day)| QueryAtom::Since {
            year,
            month: month.unwrap_or(1),
            day: day.unwrap_or(1),
        })
        .parse_next(input)
}

#[derive(Debug, Clone)]
pub enum Comparator {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

fn parse_compare_operator(input: &mut &str) -> Result<Comparator> {
    (alt(('<', '>')), opt('='))
        .map(|(relation, equals)| match relation {
            '<' if equals.is_some() => Comparator::LessOrEqual,
            '<' => Comparator::Less,
            '>' if equals.is_some() => Comparator::GreaterOrEqual,
            _ => Comparator::Greater,
        })
        .parse_next(input)
}

fn parse_region<'a>(input: &mut &'a str) -> Result<Stats<'a>> {
    preceded(opt(Caseless("alt-")), take_while(2, char::is_alphabetic))
        .take()
        .map(Stats::Region)
        .parse_next(input)
}

fn parse_percentage<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        terminated(parse_compare_operator, space0),
        terminated(float, '%'),
        opt(preceded((space1, Caseless("in"), space1), parse_region)),
    )
        .map(|(comparator, value, stats)| QueryAtom::Percentage {
            comparator,
            popularity: value,
            stats: stats.unwrap_or(Stats::Global),
        })
        .parse_next(input)
}

fn parse_cover<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        preceded(
            terminated(Caseless("cover"), space1),
            terminated(float, '%'),
        ),
        opt(preceded((space1, Caseless("in"), space1), parse_region)),
    )
        .map(|(value, stats)| QueryAtom::Cover {
            coverage: value,
            stats: stats.unwrap_or(Stats::Global),
        })
        .parse_next(input)
}

fn parse_supports<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    separated_pair(
        opt(terminated(
            alt((
                Caseless("fully").map(|_| SupportKind::Fully),
                Caseless("partially").map(|_| SupportKind::Partially),
            )),
            space1,
        )),
        terminated(Caseless("supports"), space1),
        take_while(1.., |c: char| c.is_alphanumeric() || c == '-'),
    )
    .map(|(kind, name)| QueryAtom::Supports(name, kind))
    .parse_next(input)
}

#[derive(Debug, Clone)]
pub enum VersionRange<'a> {
    Bounded(&'a str, &'a str),
    Unbounded(Comparator, &'a str),
    Accurate(&'a str),
}

fn parse_version<'a>(input: &mut &'a str) -> Result<&'a str> {
    take_while(1.., |c: char| c.is_ascii_digit() || c == '.').parse_next(input)
}

fn parse_version_range<'a>(input: &mut &'a str) -> Result<VersionRange<'a>> {
    alt((
        preceded(
            space1,
            separated_pair(parse_version, delimited(space0, '-', space0), parse_version),
        )
        .map(|(from, to)| VersionRange::Bounded(from, to)),
        preceded(
            space0,
            separated_pair(parse_compare_operator, space0, parse_version),
        )
        .map(|(comparator, version)| VersionRange::Unbounded(comparator, version)),
        preceded(space1, parse_version).map(VersionRange::Accurate),
    ))
    .parse_next(input)
}

fn parse_electron<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    preceded(Caseless("electron"), parse_version_range)
        .map(QueryAtom::Electron)
        .parse_next(input)
}

fn parse_node<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    preceded(Caseless("node"), parse_version_range)
        .map(QueryAtom::Node)
        .parse_next(input)
}

fn parse_browser<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        take_while(1.., |c: char| c.is_ascii_alphabetic() || c == '_'),
        alt((
            parse_version_range,
            preceded(space1, Caseless("tp")).map(VersionRange::Accurate),
        )),
    )
        .map(|(name, version)| QueryAtom::Browser(name, version))
        .parse_next(input)
}

fn parse_firefox_esr<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        alt((Caseless("firefox"), Caseless("fx"), Caseless("ff"))),
        space1,
        Caseless("esr"),
    )
        .map(|_| QueryAtom::FirefoxESR)
        .parse_next(input)
}

fn parse_opera_mini<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        alt((Caseless("operamini"), Caseless("op_mini"))),
        space1,
        Caseless("all"),
    )
        .map(|_| QueryAtom::OperaMini)
        .parse_next(input)
}

fn parse_current_node<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (Caseless("current"), space1, Caseless("node"))
        .map(|_| QueryAtom::CurrentNode)
        .parse_next(input)
}

fn parse_maintained_node<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    (
        Caseless("maintained"),
        space1,
        Caseless("node"),
        space1,
        Caseless("versions"),
    )
        .map(|_| QueryAtom::MaintainedNode)
        .parse_next(input)
}

fn parse_phantom<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    preceded(
        terminated(Caseless("phantomjs"), space1),
        alt(("1.9", "2.1")),
    )
    .map(|version| QueryAtom::Phantom(version == "2.1"))
    .parse_next(input)
}

fn parse_browserslist_config<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    Caseless("browserslist config")
        .map(|_| QueryAtom::BrowserslistConfig)
        .parse_next(input)
}

fn parse_defaults<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    Caseless("defaults")
        .map(|_| QueryAtom::Defaults)
        .parse_next(input)
}

fn parse_dead<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    Caseless("dead").map(|_| QueryAtom::Dead).parse_next(input)
}

fn parse_extends<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    preceded(
        terminated(Caseless("extends"), space1),
        take_while(1.., |c: char| {
            c.is_alphanumeric() || c == '-' || c == '_' || c == '@' || c == '/' || c == '.'
        }),
    )
    .map(QueryAtom::Extends)
    .parse_next(input)
}

fn parse_date_yyyy_mm_dd<'a>(input: &mut &'a str) -> Result<&'a str> {
    (
        take_while(4, |c: char| c.is_ascii_digit()),
        '-',
        take_while(2, |c: char| c.is_ascii_digit()),
        '-',
        take_while(2, |c: char| c.is_ascii_digit()),
    )
        .take()
        .parse_next(input)
}

fn parse_baseline<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    // Grammar (case-insensitive):
    //   baseline YEAR
    //   baseline widely available [on YYYY-MM-DD]
    //   baseline newly available
    //   … optionally followed by "with downstream", then optionally "including kaios"
    (
        (Caseless("baseline"), space1),
        alt((
            dec_uint.map(BaselineKind::Year),
            preceded(
                (
                    terminated(Caseless("widely"), space1),
                    Caseless("available"),
                ),
                opt(preceded(
                    (space1, Caseless("on"), space1),
                    parse_date_yyyy_mm_dd,
                )),
            )
            .map(|date| match date {
                Some(date) => BaselineKind::WidelyAvailableOnDate(date),
                None => BaselineKind::WidelyAvailable,
            }),
            (Caseless("newly"), space1, Caseless("available"))
                .map(|_| BaselineKind::NewlyAvailable),
        )),
        opt((space1, Caseless("with"), space1, Caseless("downstream"))),
        opt((space1, Caseless("including"), space1, Caseless("kaios"))),
    )
        .map(|(_, kind, downstream, kaios)| QueryAtom::Baseline {
            kind,
            downstream: downstream.is_some(),
            kaios: kaios.is_some(),
        })
        .parse_next(input)
}

fn parse_unknown<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    repeat_till::<_, _, (), _, _, _, _>(1.., any, peek(parse_composition_operator))
        .take()
        .map(QueryAtom::Unknown)
        .parse_next(input)
}

fn parse_query_atom<'a>(input: &mut &'a str) -> Result<QueryAtom<'a>> {
    alt([
        parse_last,
        parse_unreleased,
        parse_years,
        parse_since,
        parse_percentage,
        parse_cover,
        parse_supports,
        parse_electron,
        parse_node,
        parse_firefox_esr,
        parse_opera_mini,
        parse_current_node,
        parse_maintained_node,
        parse_phantom,
        parse_baseline,
        parse_browser,
        parse_browserslist_config,
        parse_defaults,
        parse_dead,
        parse_extends,
        parse_unknown,
    ])
    .parse_next(input)
}

#[derive(Debug)]
pub(crate) struct SingleQuery<'a> {
    pub(crate) raw: &'a str,
    pub(crate) atom: QueryAtom<'a>,
    pub(crate) negated: bool,
    pub(crate) is_and: bool,
}

fn parse_and(input: &mut &str) -> Result<bool> {
    delimited(space1, Caseless("and"), space1)
        .map(|_| true)
        .parse_next(input)
}

fn parse_or(input: &mut &str) -> Result<bool> {
    alt((
        delimited(space0, ',', space0).map(|_| false),
        delimited(space1, Caseless("or"), space1).map(|_| false),
    ))
    .parse_next(input)
}

fn parse_composition_operator(input: &mut &str) -> Result<bool> {
    alt((parse_and, parse_or)).parse_next(input)
}

fn parse_single_query<'a>(input: &mut &'a str) -> Result<SingleQuery<'a>> {
    (
        parse_composition_operator,
        (opt(terminated(Caseless("not"), space1)), parse_query_atom).with_taken(),
    )
        .map(|(is_and, ((negated, atom), raw))| SingleQuery {
            raw,
            atom,
            negated: negated.is_some(),
            is_and,
        })
        .parse_next(input)
}

pub(crate) fn parse_browserslist_query<'a>(input: &mut &'a str) -> Result<Vec<SingleQuery<'a>>> {
    if input.trim().is_empty() {
        return Ok(vec![]);
    }

    (
        (
            // this isn't allowed, but for better error report
            opt(terminated(Caseless("not"), space1)),
            parse_query_atom,
        )
            .with_taken(),
        repeat(0.., parse_single_query),
    )
        .map(
            |(((negated, first), first_raw), mut queries): (_, Vec<_>)| {
                queries.insert(
                    0,
                    SingleQuery {
                        raw: first_raw,
                        atom: first,
                        negated: negated.is_some(),
                        is_and: false,
                    },
                );
                queries
            },
        )
        .parse_next(input)
}

pub(crate) fn parse_electron_version(version: &str) -> Result<f32, crate::error::Error> {
    terminated(
        float::<_, _, ContextError>,
        opt(('.', digit1.verify_map(|s: &str| s.parse::<u16>().ok()))),
    )
    .parse(version)
    .map_err(|_| crate::error::Error::UnknownElectronVersion(version.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::{opts::Opts, test::run_compare};
    use test_case::test_case;

    #[test_case(""; "empty")]
    #[test_case("ie >= 6, ie <= 7"; "comma")]
    #[test_case("ie >= 6 and ie <= 7"; "and")]
    #[test_case("ie < 11 and not ie 7"; "and with not")]
    #[test_case("last 1 Baidu version and not <2%"; "with not and one-version browsers as and query")]
    #[test_case("ie >= 6 or ie <= 7"; "or")]
    #[test_case("ie < 11 or not ie 7"; "or with not")]
    #[test_case("last 2 versions and > 1%"; "swc issue 4871")]
    fn valid(query: &str) {
        run_compare(query, &Opts::default(), None);
    }
}
