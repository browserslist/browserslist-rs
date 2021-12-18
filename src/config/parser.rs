use super::PartialConfig;
use crate::error::Error;
use ahash::AHashSet;

pub(crate) fn parse<S: AsRef<str>>(
    source: &str,
    env: S,
    throw_on_missing: bool,
) -> Result<PartialConfig, Error> {
    let env = env.as_ref();
    let mut encountered_sections = AHashSet::new();
    let mut current_section = Some("defaults");

    let config = source
        .lines()
        .map(|line| {
            if let Some(index) = line.find('#') {
                &line[..index]
            } else {
                line
            }
        })
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .try_fold(
            (Vec::new(), Option::<Vec<String>>::None),
            |(mut defaults_queries, mut env_queries), line| {
                if line.starts_with('[') && line.ends_with(']') {
                    let sections = line
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(' ')
                        .filter(|env| !env.is_empty())
                        .collect::<Vec<_>>();
                    current_section = sections.iter().find(|section| **section == env).copied();
                    for section in sections {
                        if encountered_sections.contains(section) {
                            return Err(Error::DuplicatedSection(section.to_string()));
                        } else {
                            encountered_sections.insert(section);
                        }
                    }
                    Ok((
                        defaults_queries,
                        if env_queries.is_some() {
                            // we've collected queries of current env, so return it as-is
                            env_queries
                        } else if encountered_sections.contains(env) {
                            // get ready for collecting queries of current env
                            Some(vec![])
                        } else {
                            None
                        },
                    ))
                } else {
                    if current_section.is_some() {
                        // if env queries are prepared, we should add queries to them, not the "defaults"
                        if let Some(env_queries) = env_queries.as_mut() {
                            env_queries.push(line.to_string());
                        } else {
                            defaults_queries.push(line.to_string());
                        }
                    }
                    Ok((defaults_queries, env_queries))
                }
            },
        )
        .map(|(defaults, env)| PartialConfig { defaults, env });

    if throw_on_missing && env != "defaults" && !encountered_sections.contains(env) {
        Err(Error::MissingEnv(env.to_string()))
    } else {
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let source = "  \t  \n  \r\n  # comment ";
        let config = parse(source, "production", false).unwrap();
        assert!(config.defaults.is_empty());
        assert!(config.env.is_none());
    }

    #[test]
    fn no_sections() {
        let source = r"
last 2 versions
not dead
";
        let config = parse(source, "production", false).unwrap();
        assert_eq!(&*config.defaults, ["last 2 versions", "not dead"]);
        assert!(config.env.is_none());
    }

    #[test]
    fn single_line() {
        let source = r"last 2 versions, not dead";
        let config = parse(source, "production", false).unwrap();
        assert_eq!(&*config.defaults, ["last 2 versions, not dead"]);
        assert!(config.env.is_none());
    }

    #[test]
    fn empty_lines() {
        let source = r"
last 2 versions


not dead
";
        let config = parse(source, "production", false).unwrap();
        assert_eq!(&*config.defaults, ["last 2 versions", "not dead"]);
        assert!(config.env.is_none());
    }

    #[test]
    fn comments() {
        let source = r"
last 2 versions  #trailing comment
#line comment
not dead
";
        let config = parse(source, "production", false).unwrap();
        assert_eq!(&*config.defaults, ["last 2 versions", "not dead"]);
        assert!(config.env.is_none());
    }

    #[test]
    fn spaces() {
        let source = "    last 2 versions     \n  not dead    ";
        let config = parse(source, "production", false).unwrap();
        assert_eq!(&*config.defaults, ["last 2 versions", "not dead"]);
        assert!(config.env.is_none());
    }

    #[test]
    fn one_section() {
        let source = r"
[production]
last 2 versions
not dead
";
        let config = parse(source, "production", false).unwrap();
        assert!(config.defaults.is_empty());
        assert_eq!(
            config.env.as_deref().unwrap(),
            ["last 2 versions", "not dead"]
        );
    }

    #[test]
    fn defaults_and_env_mixed() {
        let source = r"
> 1%

[production]
last 2 versions
not dead
";
        let config = parse(source, "production", false).unwrap();
        assert_eq!(&*config.defaults, ["> 1%"]);
        assert_eq!(
            config.env.as_deref().unwrap(),
            ["last 2 versions", "not dead"]
        );
    }

    #[test]
    fn multi_sections() {
        let source = r"
[production]
> 1%
ie 10

[  modern]
last 1 chrome version
last 1 firefox version

[ssr  ]
node 12
";
        let config = parse(source, "production", false).unwrap();
        assert!(config.defaults.is_empty());
        assert_eq!(config.env.as_deref().unwrap(), ["> 1%", "ie 10"]);

        let config = parse(source, "modern", false).unwrap();
        assert!(config.defaults.is_empty());
        assert_eq!(
            config.env.as_deref().unwrap(),
            ["last 1 chrome version", "last 1 firefox version"]
        );

        let config = parse(source, "ssr", false).unwrap();
        assert!(config.defaults.is_empty());
        assert_eq!(config.env.as_deref().unwrap(), ["node 12"]);
    }

    #[test]
    fn shared_multi_sections() {
        let source = r"
[production   development]
> 1%
ie 10
";
        let config = parse(source, "development", false).unwrap();
        assert!(config.defaults.is_empty());
        assert_eq!(config.env.as_deref().unwrap(), ["> 1%", "ie 10"]);
    }

    #[test]
    fn duplicated_sections() {
        let source = r"
[production production]
> 1%
ie 10
";
        assert_eq!(
            parse(source, "testing", false),
            Err(Error::DuplicatedSection("production".into()))
        );

        let source = r"
[development]
last 1 chrome version

[production]
> 1 %
not dead

[development]
last 1 firefox version
";
        assert_eq!(
            parse(source, "testing", false),
            Err(Error::DuplicatedSection("development".into()))
        );
    }

    #[test]
    fn mismatch_section() {
        let source = r"
[production]
> 1%
ie 10
";
        let config = parse(source, "development", false).unwrap();
        assert!(config.defaults.is_empty());
        assert!(config.env.is_none());
    }

    #[test]
    fn throw_on_missing_env() {
        let source = "node 16";
        let err = parse(source, "SSR", true).unwrap_err();
        assert_eq!(err, Error::MissingEnv("SSR".into()));
    }

    #[test]
    fn dont_throw_if_existed() {
        let source = r"
[production]
> 1%
ie 10
";
        let config = parse(source, "production", true).unwrap();
        assert!(config.defaults.is_empty());
        assert!(config.env.is_some());
    }

    #[test]
    fn dont_throw_for_defaults() {
        let source = r"
[production]
> 1%
ie 10
";
        let config = parse(source, "defaults", true).unwrap();
        assert!(config.defaults.is_empty());
        assert!(config.env.is_none());
    }
}
