use std::{fmt, io};

use crate::MANIFEST_DIR;

/// Alias for a `Result` with the error type `find_crate::Error`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error that occurred during during reading or parsing the manifest.
#[derive(Debug)]
pub struct Error(ErrorKind);

// Hiding error variants from a library's public error type to prevent
// dependency updates from becoming breaking changes.
// We can add `is_*` methods that indicate the kind of error if needed, but
// don't expose dependencies' types directly in the public API.
/// An error that occurred during during reading or parsing the manifest.
#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// The `CARGO_MANIFEST_DIR` environment variable not found.
    NotFoundManifestDir,

    /// The manifest is invalid for the following reason.
    InvalidManifest(String),

    /// An error that occurred during opening or reading the manifest file.
    Io(io::Error),

    /// An error that occurred during parsing the manifest file.
    Toml(toml::de::Error),
}

impl Error {
    pub(crate) fn new(e: impl Into<ErrorKind>) -> Self {
        Error(e.into())
    }

    pub(crate) fn invalid_manifest(s: impl Into<String>) -> Self {
        Error(ErrorKind::InvalidManifest(s.into()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorKind::NotFoundManifestDir => {
                write!(f, "`{}` environment variable not found", MANIFEST_DIR)
            }
            ErrorKind::InvalidManifest(reason) => {
                write!(f, "The manifest is invalid because: {}", reason)
            }
            ErrorKind::Io(e) => write!(f, "an error occurred while to open or to read: {}", e),
            ErrorKind::Toml(e) => {
                write!(f, "an error occurred while parsing the manifest file: {}", e)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::Toml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error(kind)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> Self {
        ErrorKind::Io(e)
    }
}

impl From<toml::de::Error> for ErrorKind {
    fn from(e: toml::de::Error) -> Self {
        ErrorKind::Toml(e)
    }
}
