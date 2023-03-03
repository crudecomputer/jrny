use std::env;
use std::fmt;
use std::io;
use std::num;

use toml::de::Error as TomlError;

/// The canonical error type used throughout the crate.
#[derive(Debug)]
pub enum Error {
    // TODO This has gotten a bit unwieldy.
    // Should these just be individual structs now to avoid
    // big matches anywhere, or module-leel enums?
    BadEnvVar(env::VarError, String),
    ConfigNotFound(String),
    DatabaseError(postgres::Error),
    EnvNotFound,
    FileNotValid(String),
    IoError(io::Error),
    PathAlreadyExists(String),
    PathInvalid(String),
    PathNotDirectory(String),
    PathNotEmptyDirectory(String),
    RevisionNameInvalid(String),
    RevisionTimestampInvalid(num::ParseIntError, String),
    RevisionTimestampOutOfRange(String),
    RevisionsFailedReview {
        changed: usize,
        duplicate_ids: usize,
        missing: usize,
        predate_applied: usize,
    },
    TomlInvalid(TomlError, String),
    TransactionCommandFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            BadEnvVar(err, var_name) => {
                write!(f, "{} - {}", err, var_name)
            }
            ConfigNotFound(pathstr) => {
                write!(f, "`{}` not found - run in directory with `jrny.toml` file or specify path to config with `-c /path/to/config`", pathstr)
            }
            DatabaseError(err) => {
                write!(f, "{}", err)
            }
            EnvNotFound => {
                write!(f, "`jrny-env.toml` must exist within same directory as config file or `--database-url` must be provided")
            }
            FileNotValid(pathstr) => {
                write!(f, "`{}` is not a valid file", pathstr)
            }
            IoError(err) => {
                write!(f, "{}", err)
            }
            PathAlreadyExists(pathstr) => {
                write!(f, "`{}` already exists", pathstr)
            }
            PathInvalid(pathstr) => {
                write!(f, "`{}` is not a valid path", pathstr)
            }
            PathNotDirectory(pathstr) => {
                write!(f, "`{}` is not a directory", pathstr)
            }
            PathNotEmptyDirectory(pathstr) => {
                write!(f, "`{}` is not an empty directory", pathstr)
            }
            RevisionNameInvalid(filename) => {
                write!(
                    f,
                    "Invalid revision name `{}`: expected `[id].[timestamp].[name].sql` eg. `001.1618370298.my-first-revision.sql`",
                    filename
                )
            }
            RevisionTimestampInvalid(err, filename) => {
                write!(f, "Invalid revision timestamp `{}`: {}", filename, err)
            }
            RevisionTimestampOutOfRange(filename) => {
                write!(
                    f,
                    "Invalid revision timestamp `{}`: timestamp out of range",
                    filename
                )
            }
            RevisionsFailedReview {
                changed,
                duplicate_ids,
                missing,
                predate_applied,
            } => {
                let mut errs = String::new();

                if *changed > 0 {
                    errs.push_str(&format!("\n\t{} changed since being applied", changed));
                }

                if *duplicate_ids > 0 {
                    let (verb, id) = if *duplicate_ids > 1 {
                        ("have", "ids")
                    } else {
                        ("has a", "id")
                    };
                    errs.push_str(&format!("\n\t{} {} duplicate {}", duplicate_ids, verb, id));
                }

                if *missing > 0 {
                    errs.push_str(&format!("\n\t{} applied no longer present", missing));
                }

                if *predate_applied > 0 {
                    errs.push_str(&format!(
                        "\n\t{} pending occur before applied revisions",
                        predate_applied
                    ));
                }

                write!(f, "Revisions review failed:{}", errs)
            }
            TomlInvalid(err, pathstr) => {
                write!(f, "`{}` is invalid - {}", pathstr, err)
            }
            TransactionCommandFound(cmd) => {
                write!(f, "Cannot use transaction commands: found `{}`", cmd)
            }
        }
    }
}

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self {
        Self::DatabaseError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
