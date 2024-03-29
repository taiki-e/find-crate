// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::fmt;
use std::io;

use crate::MANIFEST_DIR;

// TODO: Hide error variants from a library's public error type to prevent
// dependency updates from becoming breaking changes.
/// An error that occurred when getting manifest.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The [`CARGO_MANIFEST_DIR`] environment variable not found.
    ///
    /// [`CARGO_MANIFEST_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    NotFoundManifestDir,

    /// The manifest is invalid for the following reason.
    InvalidManifest(String),

    /// The crate with the specified name not found. This error occurs only from [`find_crate`].
    ///
    /// [`find_crate`]: super::find_crate
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
                write!(f, "`{MANIFEST_DIR}` environment variable not found")
            }
            Error::InvalidManifest(reason) => {
                write!(f, "The manifest is invalid because: {reason}")
            }
            Error::NotFound => {
                f.write_str("the crate with the specified name not found in dependencies")
            }
            Error::Io(e) => write!(f, "an error occurred while to open or to read: {e}"),
            Error::Toml(e) => write!(f, "an error occurred while parsing the manifest file: {e}"),
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
