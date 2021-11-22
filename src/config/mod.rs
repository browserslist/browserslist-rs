use crate::{error::Error, opts::Opts};
use either::Either;
use parser::parse;
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

mod parser;

type Config = HashMap<String, Vec<String>>;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) struct PartialConfig {
    defaults: Vec<String>,
    env: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(untagged)]
enum PkgConfig {
    Str(String),
    Arr(Vec<String>),
    Obj(Config),
}

impl Default for PkgConfig {
    fn default() -> Self {
        Self::Obj(HashMap::default())
    }
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct PackageJson {
    browserslist: Option<PkgConfig>,
}

const ERR_DUP_PLAIN: &str = "'browserslist' file";
const ERR_DUP_RC: &str = "'.browserslistrc' file";
const ERR_DUP_PKG: &str = "'package.json' file with `browserslist` field";

pub fn load(opts: &Opts) -> Result<Vec<String>, Error> {
    if let Ok(query) = env::var("BROWSERSLIST") {
        Ok(vec![query])
    } else if let Some(config_path) = opts
        .config
        .as_ref()
        .map(Cow::from)
        .or_else(|| env::var("BROWSERSLIST_CONFIG").ok().map(Cow::from))
        .as_deref()
    {
        let config_path = Path::new(config_path);
        match config_path.file_name() {
            Some(file_name) if file_name == "package.json" => {
                let content = fs::read(config_path)
                    .map_err(|_| Error::FailedToReadConfig(format!("{}", config_path.display())))?;
                let pkg: PackageJson = serde_json::from_slice(&content)
                    .map_err(|_| Error::FailedToReadConfig(format!("{}", config_path.display())))?;
                Ok(pick_queries_by_env(
                    pkg.browserslist.ok_or_else(|| {
                        Error::MissingFieldInPkg(format!("{}", config_path.display()))
                    })?,
                    &get_env(opts),
                ))
            }
            _ => {
                let content = fs::read_to_string(config_path)
                    .map_err(|_| Error::FailedToReadConfig(format!("{}", config_path.display())))?;
                let config = parse(&content, get_env(opts))?;
                Ok(config.env.unwrap_or(config.defaults))
            }
        }
    } else {
        let path = match &opts.path {
            Some(path) => PathBuf::from(path),
            None => env::current_dir().map_err(|_| Error::FailedToAccessCurrentDir)?,
        };
        match find_config(path)? {
            Either::Left(s) => {
                let config = parse(&s, get_env(opts))?;
                Ok(config.env.unwrap_or(config.defaults))
            }
            Either::Right(config) => Ok(pick_queries_by_env(config, &get_env(opts))),
        }
    }
}

fn find_config<P: AsRef<Path>>(path: P) -> Result<Either<String, PkgConfig>, Error> {
    for dir in path.as_ref().ancestors() {
        let path_plain = dir.join("browserslist");
        let plain = File::open(&path_plain);
        let is_plain_existed = if let Ok(file) = &plain {
            file.metadata()
                .map(|metadata| metadata.is_file())
                .unwrap_or_default()
        } else {
            false
        };

        let path_rc = dir.join(".browserslistrc");
        let rc = File::open(&path_rc);
        let is_rc_existed = if let Ok(file) = &rc {
            file.metadata()
                .map(|metadata| metadata.is_file())
                .unwrap_or_default()
        } else {
            false
        };

        let path_pkg = dir.join("package.json");
        let pkg = File::open(&path_pkg)
            .ok()
            .and_then(|file| {
                if file.metadata().ok()?.is_file() {
                    serde_json::from_reader::<_, PackageJson>(BufReader::new(file)).ok()
                } else {
                    None
                }
            })
            .and_then(|json| json.browserslist);

        match (plain, rc, pkg) {
            (Ok(_), Ok(_), _) if is_plain_existed && is_rc_existed => {
                return Err(Error::DuplicatedConfig(
                    format!("{}", dir.display()),
                    ERR_DUP_PLAIN,
                    ERR_DUP_RC,
                ));
            }
            (Ok(_), _, Some(_)) if is_plain_existed => {
                return Err(Error::DuplicatedConfig(
                    format!("{}", dir.display()),
                    ERR_DUP_PLAIN,
                    ERR_DUP_PKG,
                ));
            }
            (Ok(mut plain), _, _) if is_plain_existed => {
                let mut content = String::new();
                plain
                    .read_to_string(&mut content)
                    .map_err(|_| Error::FailedToReadConfig(format!("{}", path_plain.display())))?;
                return Ok(Either::Left(content));
            }
            (_, Ok(_), Some(_)) if is_rc_existed => {
                return Err(Error::DuplicatedConfig(
                    format!("{}", dir.display()),
                    ERR_DUP_RC,
                    ERR_DUP_PKG,
                ));
            }
            (_, Ok(mut rc), _) if is_rc_existed => {
                let mut content = String::new();
                rc.read_to_string(&mut content)
                    .map_err(|_| Error::FailedToReadConfig(format!("{}", path_rc.display())))?;
                return Ok(Either::Left(content));
            }
            (_, _, Some(pkg)) => return Ok(Either::Right(pkg)),
            _ => continue,
        };
    }

    Ok(Either::Right(Default::default()))
}

fn get_env(opts: &Opts) -> Cow<str> {
    opts.env
        .as_ref()
        .map(Cow::from)
        .or_else(|| env::var("BROWSERSLIST_ENV").ok().map(Cow::from))
        .or_else(|| env::var("NODE_ENV").ok().map(Cow::from))
        .unwrap_or_else(|| Cow::from("production"))
}

fn pick_queries_by_env(config: PkgConfig, env: &str) -> Vec<String> {
    match config {
        PkgConfig::Str(query) => vec![query],
        PkgConfig::Arr(queries) => queries,
        PkgConfig::Obj(mut config) => config
            .remove(env)
            .or_else(|| config.remove("defaults"))
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env::{remove_var, set_var, temp_dir},
        fs,
    };

    #[test]
    fn load_config() {
        assert!(load(&Opts::new()).unwrap().is_empty());

        // read queries from env
        set_var("BROWSERSLIST", "last 2 versions");
        assert_eq!(&*load(&Opts::new()).unwrap(), ["last 2 versions"]);
        remove_var("BROWSERSLIST");

        // specify config file by env
        let tmp = temp_dir().join("browserslist");
        set_var("BROWSERSLIST_CONFIG", &tmp);

        assert_eq!(
            load(&Opts::new()).unwrap_err(),
            Error::FailedToReadConfig(format!("{}", tmp.display()))
        );

        fs::write(&tmp, "chrome > 90").unwrap();
        assert_eq!(load(&Opts::new()).as_deref().unwrap(), ["chrome > 90"]);
        // options `config` should have higher priority than environment variable
        set_var("BROWSERSLIST_CONFIG", "./browserslist");

        // specify config file by options
        fs::write(&tmp, "firefox > 90").unwrap();
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["firefox > 90"]
        );
        fs::remove_file(&tmp).unwrap();

        // package.json with single string format
        let tmp = temp_dir().join("package.json");
        fs::write(
            &tmp,
            &serde_json::to_string(&PackageJson {
                browserslist: Some(PkgConfig::Str("node > 10".into())),
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["node > 10"]
        );

        // package.json with array format
        fs::write(
            &tmp,
            &serde_json::to_string(&PackageJson {
                browserslist: Some(PkgConfig::Arr(vec!["node > 7.4".to_string()])),
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["node > 7.4"]
        );

        // package.json with object format
        let mut config_obj = HashMap::new();
        let _ = config_obj.insert("production".into(), vec!["> 1%".into(), "not dead".into()]);
        let _ = config_obj.insert("modern".into(), vec!["last 1 version".into()]);
        let _ = config_obj.insert("xp".into(), vec!["chrome >= 49".into()]);
        let _ = config_obj.insert("ssr".into(), vec!["node >= 12".into()]);
        fs::write(
            &tmp,
            &serde_json::to_string(&PackageJson {
                browserslist: Some(PkgConfig::Obj(config_obj)),
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["> 1%", "not dead"]
        );

        // pick queries by env
        set_var("BROWSERSLIST_ENV", "modern");
        set_var("NODE_ENV", "ssr");
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()).env("xp"))
                .as_deref()
                .unwrap(),
            ["chrome >= 49"]
        );
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["last 1 version"]
        );
        remove_var("BROWSERSLIST_ENV");
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["node >= 12"]
        );
        remove_var("NODE_ENV");

        let tmp = temp_dir().join("browserslist");
        fs::write(
            &tmp,
            r"
[development]
last 1 version

[production]
> 1%, not dead
        ",
        )
        .unwrap();
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()).env("development"))
                .as_deref()
                .unwrap(),
            ["last 1 version"]
        );

        fs::write(&tmp, "> 1%, not dead").unwrap();
        assert_eq!(
            load(Opts::new().config(tmp.to_str().unwrap()).env("development"))
                .as_deref()
                .unwrap(),
            ["> 1%, not dead"]
        );

        remove_var("BROWSERSLIST_CONFIG");

        // find configuration file
        let tmp_dir = temp_dir();
        let tmp = tmp_dir.to_str().unwrap();
        assert_eq!(
            load(Opts::new().path(&tmp)).unwrap_err(),
            Error::DuplicatedConfig(tmp.to_string(), ERR_DUP_PLAIN, ERR_DUP_PKG)
        );

        fs::write(tmp_dir.join(".browserslistrc"), "electron > 12.0").unwrap();
        assert_eq!(
            load(Opts::new().path(&tmp)).unwrap_err(),
            Error::DuplicatedConfig(tmp.to_string(), ERR_DUP_PLAIN, ERR_DUP_RC)
        );

        fs::remove_file(tmp_dir.join("browserslist")).unwrap();
        assert_eq!(
            load(Opts::new().path(&tmp)).unwrap_err(),
            Error::DuplicatedConfig(tmp.to_string(), ERR_DUP_RC, ERR_DUP_PKG)
        );

        let tmp_dir = tmp_dir.join("browserslist/1/2/3");
        fs::create_dir_all(&tmp_dir).unwrap();

        fs::write(temp_dir().join("browserslist/1/browserslist"), "node >= 16").unwrap();
        assert_eq!(
            load(Opts::new().path(tmp_dir.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["node >= 16"]
        );

        fs::write(temp_dir().join("browserslist/1/2/package.json"), "{}").unwrap();
        assert_eq!(
            load(Opts::new().path(tmp_dir.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["node >= 16"]
        );

        let tmp = temp_dir();
        fs::remove_file(tmp.join("package.json")).unwrap();
        fs::remove_file(tmp.join("browserslist/1/2/package.json")).unwrap();
        fs::remove_file(tmp.join("browserslist/1/browserslist")).unwrap();
        assert_eq!(
            load(Opts::new().path(tmp_dir.to_str().unwrap()))
                .as_deref()
                .unwrap(),
            ["electron > 12.0"]
        );

        fs::remove_dir_all(tmp.join("browserslist")).unwrap();

        // load config from current directory if no options set
        assert!(load(&Opts::new()).unwrap().is_empty());
        let original_cwd = env::current_dir().unwrap();
        fs::write(tmp.join(".browserslistrc"), "not dead").unwrap();
        env::set_current_dir(&tmp).unwrap();
        assert_eq!(load(&Opts::new()).as_deref().unwrap(), ["not dead"]);
        env::set_current_dir(original_cwd).unwrap();

        fs::remove_file(tmp.join(".browserslistrc")).unwrap();
    }
}
