use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorKind {
    MissingId,
    ParentNotFound,
    IdNotFound,
    ParentNotAvailable,
    FailedToBorrowCategory,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: &str) -> Error {
        Error {
            kind,
            message: message.to_string(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<cooplan_definitions_lib::error::Error> for Error {
    fn from(error: cooplan_definitions_lib::error::Error) -> Self {
        let kind: ErrorKind = match error.kind() {
            cooplan_definitions_lib::error::ErrorKind::FailedToBorrowCategory => {
                ErrorKind::FailedToBorrowCategory
            }
            cooplan_definitions_lib::error::ErrorKind::MissingId => ErrorKind::MissingId,
            cooplan_definitions_lib::error::ErrorKind::ParentNotAvailable => {
                ErrorKind::ParentNotAvailable
            }
        };

        Error {
            kind,
            message: error.message.clone(),
        }
    }
}
