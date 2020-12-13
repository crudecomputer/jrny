use std::{env, fmt, io, num};
use toml::de::Error as TomlError;

pub enum Error {
    BadEnvVar(env::VarError),
    ConfigNotFound(String),
    ConfigNotFile(String),
    ConfigInvalid(TomlError, String),
    DatabaseError(postgres::Error),
    FileNotValid(String),
    IoError(io::Error),
    PathAlreadyExists(String),
    PathInvalid(String),
    PathNotDirectory(String),
    PathNotEmptyDirectory(String),
    RevisionNameInvalid(String),
    RevisionTimestampInvalid(num::ParseIntError, String),
    RevisionsFailedReview { changed: usize, missing: usize },
    TransactionCommandFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            BadEnvVar(err) => {
                write!(f, "{}", err)
            }
            ConfigNotFound(pathstr) => {
                write!(f, "`{}` not found - run in directory with `jrny.toml` file or specify path to config with `-c /path/to/config`", pathstr)
            }
            ConfigNotFile(pathstr) => {
                write!(f, "`{}` must be a valid file", pathstr)
            }
            ConfigInvalid(err, pathstr) => {
                write!(f, "`{}` is invalid - {}", pathstr, err)
            }
            DatabaseError(err) => {
                write!(f, "{}", err)
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
                write!(f, "Invalid revision name `{}`: expected [timestamp].[name].sql", filename)
            }
            RevisionTimestampInvalid(err, filename) => {
                write!(f, "Invalid revision timestamp `{}`: {}", filename, err)
            }
            RevisionsFailedReview { changed, missing } => {
                let mut errs = String::new();

                if *changed > 0 {
                    errs.push_str(&format!(
                        "\n\t{} changed since being applied",
                        changed,
                    ));
                }

                if *missing > 0 {
                    errs.push_str(&format!(
                        "\n\t{} no longer present on disk",
                        missing,
                    ));
                }

                write!(f, "Revisions review failed:{}", errs)
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

impl From<env::VarError> for Error {
   fn from(e: env::VarError) -> Self {
       Self::BadEnvVar(e)
   } 
}

impl From<io::Error> for Error {
   fn from(e: io::Error) -> Self {
       Self::IoError(e)
   } 
}
