use std::env;
use std::fmt;
use std::io;
use std::num;

use toml::de::Error as TomlError;

use crate::commands::RevisionSummary;

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
    RevisionsFailedReview(RevisionSummary),
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
            RevisionsFailedReview(summary) => {
                let mut errs = String::new();
                let sol = "\n  -";

                if summary.files_changed > 0 {
                    errs.push_str(&match summary.files_changed {
                        1 => format!("{sol} 1 revision has been changed after being applied"),
                        count => format!("{sol} {count} revisions have changed after being applied")
                    });
                }

                // It takes at least two revisions to have "duplicate" ids
                if summary.duplicate_ids > 0 {
                    errs.push_str(&format!("{sol} {} revisions have duplicate ids", summary.duplicate_ids));
                }

                if summary.files_not_found > 0 {
                    errs.push_str(&match summary.files_not_found {
                        1 => format!("{sol} 1 revision file could not be found"),
                        count => format!("{sol} {count} revision files could not be found")
                    });
                }

                if summary.preceding_applied > 0 {
                    errs.push_str(&match summary.preceding_applied {
                        1 => format!("{sol} 1 pending revision occurs before applied revisions"),
                        count => format!("{sol} {count} pending revisions occur before applied revisions")
                    });
                }

                write!(f, "The journey has problems:{}", errs)
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
