//! Find the crate name from the current `Cargo.toml`.
//!
//! When writing declarative macros, `$crate` representing the current crate is
//! very useful, but procedural macros do not have this. If you know the current
//! name of the crate you want to use, you can do the same thing as `$crate`.
//! This crate provides the features to make it easy.
//!
//! Note: This crate is intended to provide more powerful features such as
//! support for multiple crate names and versions. For general purposes,
//! [proc-macro-crate], which provides a simpler API, may be easier to use.
//!
//! [proc-macro-crate]: https://github.com/bkchr/proc-macro-crate
//!
//! # Examples
//!
//! [`find_crate()`] gets the crate name from `Cargo.toml`.
//!
//! ```rust
//! use find_crate::find_crate;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::quote;
//!
//! fn import() -> TokenStream {
//!     let name = find_crate(|s| s == "foo").unwrap().name;
//!     let name = Ident::new(&name, Span::call_site());
//!     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
//!     quote!(extern crate #name as _foo;)
//! }
//! ```
//!
//! As in this example, it is easy to handle cases where proc-macro is exported
//! from multiple crates.
//!
//! ```rust
//! use find_crate::find_crate;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::quote;
//!
//! fn import() -> TokenStream {
//!     let name = find_crate(|s| s == "foo" || s == "foo-core").unwrap().name;
//!     let name = Ident::new(&name, Span::call_site());
//!     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
//!     quote!(extern crate #name as _foo;)
//! }
//! ```
//!
//! Using [`Manifest`] to search for multiple crates. It is much more efficient
//! than using `find_crate()` for each crate.
//!
//! ```rust
//! use find_crate::Manifest;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::{format_ident, quote};
//!
//! const CRATE_NAMES: &[&[&str]] = &[
//!     &["foo", "foo-core"],
//!     &["bar", "bar-util", "bar-core"],
//!     &["baz"],
//! ];
//!
//! fn imports() -> TokenStream {
//!     let mut tts = TokenStream::new();
//!     let manifest = Manifest::new().unwrap();
//!
//!     for names in CRATE_NAMES {
//!         let name = manifest.find(|s| names.iter().any(|x| s == *x)).unwrap().name;
//!         let name = Ident::new(&name, Span::call_site());
//!         let import_name = format_ident!("_{}", names[0]);
//!         // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
//!         tts.extend(quote!(extern crate #name as #import_name;));
//!     }
//!     tts
//! }
//! ```
//!
//! By default it will be searched from `dependencies` and `dev-dependencies`.
//! Also, [`find_crate()`] and [`Manifest::new()`] read `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.

#![doc(html_root_url = "https://docs.rs/find-crate/0.5.0")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    single_use_lifetimes,
    unreachable_pub
)]
#![warn(clippy::all, clippy::default_trait_access)]
// mem::take and #[non_exhaustive] requires Rust 1.40
#![allow(clippy::mem_replace_with_default, clippy::manual_non_exhaustive)]

use std::{
    env, fmt,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};
use toml::value::{Table, Value};

type Result<T> = std::result::Result<T, Error>;

/// `CARGO_MANIFEST_DIR` environment variable.
const MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

/// Find the crate name from the current `Cargo.toml`.
///
/// This function reads `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.
///
/// Note that this function needs to be used in the context of procedural macros.
///
/// # Examples
///
/// ```rust
/// use find_crate::find_crate;
/// use proc_macro2::{Ident, Span, TokenStream};
/// use quote::quote;
///
/// fn import() -> TokenStream {
///     let name = find_crate(|s| s == "foo" || s == "foo-core").unwrap().name;
///     let name = Ident::new(&name, Span::call_site());
///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
///     quote!(extern crate #name as _foo;)
/// }
/// ```
pub fn find_crate<P>(predicate: P) -> Result<Package>
where
    P: FnMut(&str) -> bool,
{
    Manifest::new()?.find(predicate).ok_or(Error::NotFound)
}

/// The kind of dependencies to be searched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dependencies {
    /// Search from `dependencies` and `dev-dependencies`.
    Default,
    /// Search from `dependencies`.
    Release,
    /// Search from `dev-dependencies`.
    Dev,
    /// Search from `build-dependencies`.
    Build,
    /// Search from `dependencies`, `dev-dependencies` and `build-dependencies`.
    All,
}

impl Dependencies {
    fn as_slice(self) -> &'static [&'static str] {
        match self {
            Dependencies::Default => &["dependencies", "dev-dependencies"],
            Dependencies::Release => &["dependencies"],
            Dependencies::Dev => &["dev-dependencies"],
            Dependencies::Build => &["build-dependencies"],
            Dependencies::All => &["dependencies", "dev-dependencies", "build-dependencies"],
        }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Dependencies::Default
    }
}

/// The package data. This has information on the current package name,
/// original package name, and specified version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// The key of this dependency in the manifest.
    key: String,

    // The key or the value of 'package' key.
    // If this is `None`, the value of `key` field is the original name.
    package: Option<String>,

    /// The current name of the package. This is always a valid rust identifier.
    pub name: String,

    /// The version requirement of the package. Returns `*` if no version
    /// requirement is specified.
    pub version: String,
}

impl Package {
    /// Returns the original package name.
    pub fn original_name(&self) -> &str {
        self.package.as_ref().unwrap_or(&self.key)
    }

    /// Returns `true` if the value of the [`name`] field is the original package
    /// name.
    ///
    /// [`name`]: Package::name
    pub fn is_original(&self) -> bool {
        self.package.is_none()
    }
}

/// The manifest of cargo.
///
/// Note that this function needs to be used in the context of procedural macros.
#[derive(Debug, Clone)]
pub struct Manifest {
    manifest: Table,

    /// The kind of dependencies to be searched.
    pub dependencies: Dependencies,
}

impl Manifest {
    /// Constructs a new `Manifest` from the current `Cargo.toml`.
    ///
    /// This function reads `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.
    pub fn new() -> Result<Self> {
        Self::from_path(&manifest_path()?)
    }

    // TODO: Should we support custom manifest paths?
    //       And what should we do if the file is not found?
    //       (should we use `CARGO_MANIFEST_DIR`? Or should we return an error?)
    /// Constructs a new `Manifest` from the specified toml file.
    fn from_path(manifest_path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        File::open(manifest_path)?.read_to_end(&mut bytes)?;
        toml::from_slice(&bytes).map_err(Into::into).map(Self::from_toml)
    }

    /// Constructs a new `Manifest` from a toml table.
    pub fn from_toml(manifest: Table) -> Self {
        Self { manifest, dependencies: Dependencies::default() }
    }

    /// Find the crate.
    ///
    /// The argument of the closure is the original name of the package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    ///
    /// fn import() -> TokenStream {
    ///     let manifest = Manifest::new().unwrap();
    ///     let name = manifest.find(|s| s == "foo" || s == "foo-core").unwrap().name;
    ///     let name = Ident::new(&name, Span::call_site());
    ///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    ///     quote!(extern crate #name as _foo;)
    /// }
    /// ```
    #[inline]
    pub fn find<P>(&self, mut predicate: P) -> Option<Package>
    where
        P: FnMut(&str) -> bool,
    {
        self.find2(|s, _| predicate(s))
    }

    /// Find the crate.
    ///
    /// The first argument of the closure is the original name of the package
    /// and the second argument is the version of the package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    /// use semver::{Version, VersionReq};
    ///
    /// fn check_version(req: &str, version: &Version) -> bool {
    ///     VersionReq::parse(req).unwrap().matches(version)
    /// }
    ///
    /// fn import() -> TokenStream {
    ///     let version = Version::parse("0.3.0").unwrap();
    ///     let manifest = Manifest::new().unwrap();
    ///     let name = manifest
    ///         .find2(|name, req| name == "foo" && check_version(req, &version))
    ///         .unwrap()
    ///         .name;
    ///     let name = Ident::new(&name, Span::call_site());
    ///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    ///     quote!(extern crate #name as _foo;)
    /// }
    /// ```
    #[inline]
    pub fn find2<P>(&self, predicate: P) -> Option<Package>
    where
        P: FnMut(&str, &str) -> bool,
    {
        find(&self.manifest, self.dependencies, predicate)
    }

    /// The package for the crate that this manifest represents.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    ///
    /// fn crate_name() -> TokenStream {
    /// }
    /// ```
    pub fn crate_package(&self) -> Result<Package> {
        let package_section = self.manifest.get("package").ok_or_else(|| {
            Error::InvalidManifest("[package] section is missing".to_string())
        })?;

        let package_key_value = package_section.get("name").ok_or_else(|| {
            Error::InvalidManifest("[package] section is missing `name`".to_string())
        })?;

        let package_key = match package_key_value {
            Value::String(name) => name,
            _ => {
                return Err(Error::InvalidManifest("`name` in [package] section is not a string".to_string()))
            }
        };

        let package_version_value = package_section.get("version").ok_or_else(|| {
            Error::InvalidManifest("[package] section is missing `version`".to_string())
        })?;

        let package_version = match package_version_value {
            Value::String(version) => version,
            _ => {
                return Err(Error::InvalidManifest("`version` in [package] section is not a string".to_string()))
            }
        };

        let package = Package {
            key: package_key.clone(),
            package: None,
            name: package_key.replace("-", "_"),
            version: package_version.clone(),
        };

        Ok(package)
    }
}

fn manifest_path() -> Result<PathBuf> {
    env::var_os(MANIFEST_DIR).ok_or(Error::NotFoundManifestDir).map(PathBuf::from).map(
        |mut path| {
            path.push("Cargo.toml");
            path
        },
    )
}

fn find<P>(manifest: &Table, dependencies: Dependencies, mut predicate: P) -> Option<Package>
where
    P: FnMut(&str, &str) -> bool,
{
    fn find_inner<P>(table: &Table, dependencies: &str, predicate: P) -> Option<Package>
    where
        P: FnMut(&str, &str) -> bool,
    {
        find_from_dependencies(table.get(dependencies)?.as_table()?, predicate)
    }

    dependencies
        .as_slice()
        .iter()
        .find_map(|dependencies| find_inner(manifest, dependencies, &mut predicate))
        .or_else(|| {
            dependencies.as_slice().iter().find_map(|dependencies| {
                manifest
                    .get("target")?
                    .as_table()?
                    .values()
                    .find_map(|table| find_inner(table.as_table()?, dependencies, &mut predicate))
            })
        })
}

fn find_from_dependencies<P>(table: &Table, mut predicate: P) -> Option<Package>
where
    P: FnMut(&str, &str) -> bool,
{
    fn package<P>(value: &Value, version: &str, predicate: P) -> Option<String>
    where
        P: FnOnce(&str, &str) -> bool,
    {
        value
            .as_table()?
            .get("package")?
            .as_str()
            .and_then(|s| if predicate(s, version) { Some(s.to_string()) } else { None })
    }

    fn version(value: &Value) -> Option<&str> {
        value.as_str().or_else(|| value.as_table()?.get("version")?.as_str())
    }

    table.iter().find_map(|(key, value)| {
        let version = version(value).unwrap_or("*");
        let package = package(value, version, &mut predicate);
        if package.is_some() || predicate(key, version) {
            Some(Package {
                key: key.clone(),
                name: key.replace("-", "_"),
                version: version.to_string(),
                package,
            })
        } else {
            None
        }
    })
}

// =================================================================================================
// Error

/// An error that occurred when getting manifest.
#[derive(Debug)]
pub enum Error {
    /// `CARGO_MANIFEST_DIR` environment variable not found.
    NotFoundManifestDir,
    /// The manifest is invalid for the following reason.
    InvalidManifest(String),
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
            Error::InvalidManifest(reason) => {
                write!(f, "The manifest is invalid because: {}", reason)
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
