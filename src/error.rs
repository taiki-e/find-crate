use std::{fmt, io};

use super::MANIFEST_DIR;

/// An error that occurred when getting manifest.
#[derive(Debug)]
pub enum Error {
    /// `CARGO_MANIFEST_DIR` environment variable not found.
    NotFoundManifestDir,
    /// The crate with the specified name not found. This error occurs only from [`find_crate()`].
    ///
    /// [`find_crate()`]: fn.find_crate.html
    NotFound,
    /// An error occurred while to open or to read the manifest file.
    Io(io::Error),
    /// An error occurred while to parse the manifest file.
    Toml(toml::de::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFoundManifestDir => {
                write!(f, "`{}` environment variable not found", MANIFEST_DIR)
            }
            Error::NotFound => {
                write!(f, "the crate with the specified name not found in dependencies")
            }
            Error::Io(e) => write!(f, "an error occurred while to open or to read: {}", e),
            Error::Toml(e) => write!(f, "an error occurred while parsing the manifest file: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Toml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Toml(e)
    }
}
