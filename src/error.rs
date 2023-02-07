use core::fmt;

use crate::MANIFEST_DIR;

pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

/// An error that occurred during loading or resolving the Cargo configuration.
#[derive(Debug)]
pub struct Error(ErrorKind);

// Hiding error variants from a library's public error type to prevent
// dependency updates from becoming breaking changes.
// We can add `is_*` methods that indicate the kind of error if needed, but
// don't expose dependencies' types directly in the public API.
#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// The `CARGO_MANIFEST_DIR` environment variable not found.
    NotFoundManifestDir,
    /// The manifest is invalid for the following reason.
    InvalidManifest(String),

    /// The crate with the specified name not found. This error occurs only from [`find_crate`].
    ///
    /// [`find_crate`]: super::find_crate
    NotFound,

    /// An error that occurred during parsing the manifest file.
    Toml(toml::de::Error),

    WithContext(String, Option<Box<dyn std::error::Error + Send + Sync + 'static>>),
}

impl Error {
    pub(crate) fn new(e: impl Into<ErrorKind>) -> Self {
        Self(e.into())
    }

    /// Returns `true` if the crate with the specified name not found. This error occurs only from [`find_crate`].
    ///
    /// [`find_crate`]: super::find_crate
    pub fn is_not_found(&self) -> bool {
        matches!(self.0, ErrorKind::NotFound)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorKind::NotFoundManifestDir => {
                write!(f, "`{MANIFEST_DIR}` environment variable not found")
            }
            ErrorKind::InvalidManifest(reason) => {
                write!(f, "The manifest is invalid because: {reason}")
            }
            ErrorKind::NotFound => {
                write!(f, "the crate with the specified name not found in dependencies")
            }
            ErrorKind::Toml(e) => {
                write!(f, "an error occurred while parsing the manifest file: {e}")
            }
            ErrorKind::WithContext(e, ..) => fmt::Display::fmt(e, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorKind::Toml(e) => e.source(),
            ErrorKind::WithContext(_, e) => Some(&**e.as_ref()?),
            _ => None,
        }
    }
}

impl From<toml::de::Error> for ErrorKind {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

// Note: Do not implement From<ThirdPartyErrorType> to prevent dependency
// updates from becoming breaking changes.
// Implementing `From<StdErrorType>` should also be avoided whenever possible,
// as it would be a breaking change to remove the implementation if the
// conversion is no longer needed due to changes in the internal implementation.

// Inspired by anyhow::Context.
pub(crate) trait Context<T, E> {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: fmt::Display;
    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: fmt::Display,
        F: FnOnce() -> C;
}
impl<T, E> Context<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: fmt::Display,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error(ErrorKind::WithContext(context.to_string(), Some(Box::new(e))))),
        }
    }
    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error(ErrorKind::WithContext(context().to_string(), Some(Box::new(e))))),
        }
    }
}
impl<T> Context<T, core::convert::Infallible> for Option<T> {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: fmt::Display,
    {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error(ErrorKind::WithContext(context.to_string(), None))),
        }
    }
    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error(ErrorKind::WithContext(context().to_string(), None))),
        }
    }
}
