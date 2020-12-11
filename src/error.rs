//use postgres;
use std::{env, fmt, io};

pub enum Error {
    AlreadyExists(String),
    BadEnvVar(env::VarError),
    DatabaseError(postgres::Error),
    InvalidPath(String),
    IoError(io::Error),
    NotDirectory(String),
    NotEmptyDirectory(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            AlreadyExists(pathstr) => {
                write!(f, "{} already exists", pathstr)
            }
            BadEnvVar(err) => {
                write!(f, "{}", err)
            }
            DatabaseError(err) => {
                write!(f, "{}", err)
            }
            InvalidPath(pathstr) => {
                write!(f, "'{}' is not a valid path", pathstr)
            }
            IoError(err) => {
                write!(f, "{}", err)
            }
            NotDirectory(pathstr) => {
                write!(f, "{} is not a directory", pathstr)
            }
            NotEmptyDirectory(pathstr) => {
                write!(f, "{} is not an empty directory", pathstr)
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
