//! Find the crate name from the current `Cargo.toml` (`$crate` for proc-macro).
//!
//! When writing declarative macros, `$crate` representing the current crate is
//! very useful, but procedural macros do not have this. If you know the current
//! name of the crate you want to use, you can do the same thing as `$crate`.
//! This crate provides the features to make it easy.
//!
//! ## Examples
//!
//! [`find_crate()`] gets the crate name from `Cargo.toml`.
//!
//! ```rust
//! use find_crate::find_crate;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::quote;
//!
//! fn import() -> TokenStream {
//!     let name = find_crate(|s| s == "foo").unwrap();
//!     let name = Ident::new(&name, Span::call_site());
//!     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
//!     quote!(extern crate #name as _foo;)
//! }
//! ```
//!
//! As in this example, it is easy to handle cases where proc-macro is exported from multiple crates.
//!
//! ```rust
//! use find_crate::find_crate;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::quote;
//!
//! fn import() -> TokenStream {
//!     let name = find_crate(|s| s == "foo" || s == "foo-core").unwrap();
//!     let name = Ident::new(&name, Span::call_site());
//!     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
//!     quote!(extern crate #name as _foo;)
//! }
//! ```
//!
//! Search for multiple crates. It is much more efficient than using
//! [`find_crate()`] for each crate.
//!
//! ```rust
//! use find_crate::Manifest;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::quote;
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
//!     let manifest = manifest.lock();
//!
//!     for names in CRATE_NAMES {
//!         let name = manifest.find_name(|s| names.iter().any(|x| s == *x)).unwrap();
//!         let name = Ident::new(&name, Span::call_site());
//!         let import_name = Ident::new(&format!("_{}", names[0]), Span::call_site());
//!         // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
//!         tts.extend(quote!(extern crate #name as #import_name;));
//!     }
//!     tts
//! }
//! ```
//!
//! By default it will be searched from `dependencies`, `dev-dependencies` and `build-dependencies`.
//! Also, [`find_crate()`] and [`Manifest::new()`] read `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.

#![doc(html_root_url = "https://docs.rs/find-crate/0.4.0")]
#![doc(test(attr(deny(warnings), allow(dead_code, unused_assignments, unused_variables))))]
#![warn(missing_docs, missing_debug_implementations)]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
// It cannot be included in the published code because these lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![warn(clippy::all, clippy::pedantic)]

use std::{
    borrow::Cow,
    env,
    error,
    fmt,
    fs::File,
    io::{self, Read as _Read}, // Rust 1.33+ => Read as _
    ops::Deref,
    path::{Path, PathBuf},
    result,
};

use toml::value::{Table, Value};

use self::Error::{NotFound, NotFoundManifestDir, NotFoundManifestFile, Open, Read, Toml};

type Result<T> = result::Result<T, Error>;

/// The kinds of dependencies searched by default.
pub const DEFAULT_DEPENDENCIES: &[&str] = &_DEFAULT_DEPENDENCIES;

// for const_err
const _DEFAULT_DEPENDENCIES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// `CARGO_MANIFEST_DIR` environment variable.
const MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

/// An error that occurred when getting manifest.
#[derive(Debug)]
pub enum Error {
    /// `CARGO_MANIFEST_DIR` environment variable not found.
    NotFoundManifestDir,
    /// `Cargo.toml` or specified manifest file not found.
    NotFoundManifestFile(PathBuf),
    /// An error occurred while to open the manifest file.
    Open(PathBuf, io::Error),
    /// An error occurred while reading the manifest file.
    Read(PathBuf, io::Error),
    /// An error occurred while parsing the manifest file.
    Toml(toml::de::Error),
    /// The crate with the specified name not found. This error occurs only from [`find_crate()`].
    ///
    /// [`find_crate()`]: fn.find_crate.html
    NotFound(PathBuf),
}

impl fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::_DEFAULT_DEPENDENCIES as D;
        match self {
            NotFoundManifestDir => write!(f, "`{}` environment variable not found", MANIFEST_DIR),
            NotFoundManifestFile(path) => write!(f, "the manifest file not found: {}", path.display()),
            Open(path, err) => write!(f, "an error occurred while to open {}: {}", path.display(), err),
            Read(path, err) => write!(f, "an error occurred while reading {}: {}", path.display(), err),
            Toml(err) => write!(f, "an error occurred while parsing the manifest file: {}", err),
            NotFound(path) => write!(f, "the crate with the specified name not found in {}, {} or {} in {}", D[0], D[1], D[2], path.display()),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Open(_, err) | Read(_, err) => Some(err),
            Toml(err) => Some(err),
            _ => None,
        }
    }
}

/// Find the crate name from the current `Cargo.toml`.
///
/// This function reads `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.
///
/// Note that this function must be used in the context of proc-macro.
///
/// ## Examples
///
/// ```rust
/// use find_crate::find_crate;
/// use proc_macro2::{Ident, Span, TokenStream};
/// use quote::quote;
///
/// fn import() -> TokenStream {
///     let name = find_crate(|s| s == "foo" || s == "foo-core").unwrap();
///     let name = Ident::new(&name, Span::call_site());
///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
///     quote!(extern crate #name as _foo;)
/// }
/// ```
pub fn find_crate<P>(predicate: P) -> Result<String>
where
    P: FnMut(&str) -> bool,
{
    let manifest_path = manifest_path()?;
    Manifest::from_path(&manifest_path)?
        .find(predicate)
        .map(|package| package.rust_ident.into_owned())
        .ok_or_else(|| NotFound(manifest_path))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FindOptions<'a> {
    /// The names of the tables to be searched
    dependencies: &'a [&'a str],

    /// Whether or not to convert the name of the retrieved crate to a valid
    /// rust identifier
    rust_ident: bool,
}

impl Default for FindOptions<'_> {
    fn default() -> Self {
        Self { dependencies: DEFAULT_DEPENDENCIES, rust_ident: true }
    }
}

/// The package data. This has information on the current package name,
/// original package name, and specified version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package<'a> {
    /// The key of this dependency in the manifest.
    key: &'a str,

    // value or version key's value
    /// The specified version of the package.
    version: Option<&'a str>,

    // key or package key's value
    /// If this is `None`, the value of `key` field is the original name.
    package: Option<&'a str>,

    /// If this is `Cow::Owned`, the value is a valid rust identifier.
    rust_ident: Cow<'a, str>,
}

impl Package<'_> {
    /// Returns the current package name.
    pub fn name(&self) -> &str {
        &self.rust_ident
    }

    /// Returns the original package name.
    pub fn original_name(&self) -> &str {
        self.package.as_ref().unwrap_or(&self.key)
    }

    /// Returns `true` if the value returned by `Package::name()` is a valid rust
    /// identifier.
    pub fn is_rust_ident(&self) -> bool {
        match &self.rust_ident {
            Cow::Borrowed(s) => !s.contains('-'),
            Cow::Owned(_) => true,
        }
    }

    /// Returns `true` if the value returned by `Package::name()` is the original
    /// package name.
    pub fn is_original(&self) -> bool {
        self.package.is_none()
    }

    /// Returns the version of the package.
    pub fn version(&self) -> Option<&str> {
        self.version.as_ref().map(Deref::deref)
    }
}

/// The manifest of cargo.
///
/// Note that this item must be used in the context of proc-macro.
#[derive(Debug, Clone)]
pub struct Manifest<'a> {
    manifest: Table,
    options: FindOptions<'a>,
}

impl<'a> Manifest<'a> {
    /// Constructs a new `Manifest` from the current `Cargo.toml`.
    ///
    /// This function reads `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.
    pub fn new() -> Result<Self> {
        Self::from_path(&manifest_path()?)
    }

    /// Constructs a new `Manifest` from the specified toml file.
    pub fn from_path(manifest_path: &Path) -> Result<Self> {
        fn open(path: &Path) -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            File::open(path)
                .map_err(|e| Open(path.to_owned(), e))?
                .read_to_end(&mut bytes)
                .map_err(|e| Read(path.to_owned(), e))
                .map(|_| bytes)
        }

        if manifest_path.is_file() {
            Self::from_bytes(&open(manifest_path)?)
        } else {
            Err(NotFoundManifestFile(manifest_path.to_owned()))
        }
    }

    /// Constructs a new `Manifest` from the bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        toml::from_slice(bytes).map_err(Toml).map(Self::from_raw)
    }

    /// Constructs a new `Manifest` from the raw manifest.
    fn from_raw(manifest: Table) -> Self {
        Manifest { manifest, options: FindOptions::default() }
    }

    /// Returns the kinds of dependencies to be searched. The default is
    /// `dependencies`, `dev-dependencies` and `build-dependencies`.
    pub fn dependencies(&self) -> &[&str] {
        self.options.dependencies
    }

    /// Sets the kinds of dependencies to be searched. The default is
    /// `dependencies`, `dev-dependencies` and `build-dependencies`.
    pub fn set_dependencies(&mut self, dependencies: &'a [&'a str]) {
        self.options.dependencies = dependencies;
    }

    /// Returns whether or not to convert the name of the retrieved crate to a
    /// valid rust identifier. The default is `true`.
    pub fn rust_ident(&self) -> bool {
        self.options.rust_ident
    }

    /// Sets whether or not to convert the name of the retrieved crate to a
    /// valid rust identifier.
    pub fn set_rust_ident(&mut self, rust_ident: bool) {
        self.options.rust_ident = rust_ident;
    }

    /// Lock the kinds of dependencies to be searched. This is more efficient when you want to
    /// search multiple times without changing the kinds of dependencies to be searched.
    pub fn lock(&self) -> ManifestLock<'_> {
        ManifestLock::new(self)
    }

    /// Find the crate name.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    ///
    /// fn import() -> TokenStream {
    ///     let manifest = Manifest::new().unwrap();
    ///     let name = manifest.find_name(|s| s == "foo" || s == "foo-core").unwrap();
    ///     let name = Ident::new(&name, Span::call_site());
    ///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    ///     quote!(extern crate #name as _foo;)
    /// }
    /// ```
    pub fn find_name<P>(&self, predicate: P) -> Option<Cow<'_, str>>
    where
        P: FnMut(&str) -> bool,
    {
        self.find(predicate).map(|package| package.rust_ident)
    }

    /// Find the crate.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    ///
    /// fn import() -> TokenStream {
    ///     let manifest = Manifest::new().unwrap();
    ///     let package = manifest.find(|s| s == "foo" || s == "foo-core").unwrap();
    ///     let name = Ident::new(package.name(), Span::call_site());
    ///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    ///     quote!(extern crate #name as _foo;)
    /// }
    /// ```
    pub fn find<P>(&self, mut predicate: P) -> Option<Package<'_>>
    where
        P: FnMut(&str) -> bool,
    {
        self.dependencies().iter().find_map(|dependencies| {
            self.manifest
                .get(*dependencies)
                .and_then(Value::as_table)
                .and_then(|t| find_from_dependencies(t, &mut predicate, self.rust_ident()))
        })
    }
}

/// A locked reference to the dependencies tables of `Manifest` to be searched.
#[derive(Debug, Clone)]
pub struct ManifestLock<'a> {
    manifest: &'a Manifest<'a>,
    tables: Vec<&'a Table>,
}

impl<'a> ManifestLock<'a> {
    fn new(manifest: &'a Manifest<'a>) -> Self {
        Self {
            tables: manifest
                .dependencies()
                .iter()
                .filter_map(|&dependencies| {
                    manifest.manifest.get(dependencies).and_then(Value::as_table)
                })
                .collect(),
            manifest,
        }
    }

    /// Find the crate name.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    ///
    /// const CRATE_NAMES: &[&[&str]] = &[
    ///     &["foo", "foo-core"],
    ///     &["bar", "bar-util", "bar-core"],
    ///     &["baz"],
    /// ];
    ///
    /// fn imports() -> TokenStream {
    ///     let mut tts = TokenStream::new();
    ///     let manifest = Manifest::new().unwrap();
    ///     let manifest = manifest.lock();
    ///
    ///     for names in CRATE_NAMES {
    ///         let name = manifest.find_name(|s| names.iter().any(|x| s == *x)).unwrap();
    ///         let name = Ident::new(&name, Span::call_site());
    ///         let import_name = Ident::new(&format!("_{}", names[0]), Span::call_site());
    ///         // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
    ///         tts.extend(quote!(extern crate #name as #import_name;));
    ///     }
    ///     tts
    /// }
    /// ```
    pub fn find_name<P>(&self, predicate: P) -> Option<Cow<'_, str>>
    where
        P: FnMut(&str) -> bool,
    {
        self.find(predicate).map(|package| package.rust_ident)
    }

    /// Find the crate.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use find_crate::Manifest;
    /// use proc_macro2::{Ident, Span, TokenStream};
    /// use quote::quote;
    ///
    /// const CRATE_NAMES: &[&[&str]] = &[
    ///     &["foo", "foo-core"],
    ///     &["bar", "bar-util", "bar-core"],
    ///     &["baz"],
    /// ];
    ///
    /// fn imports() -> TokenStream {
    ///     let mut tts = TokenStream::new();
    ///     let manifest = Manifest::new().unwrap();
    ///     let manifest = manifest.lock();
    ///
    ///     for names in CRATE_NAMES {
    ///         let package = manifest.find(|s| names.iter().any(|x| s == *x)).unwrap();
    ///         let name = Ident::new(package.name(), Span::call_site());
    ///         let import_name = Ident::new(&format!("_{}", names[0]), Span::call_site());
    ///         // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
    ///         tts.extend(quote!(extern crate #name as #import_name;));
    ///     }
    ///     tts
    /// }
    /// ```
    pub fn find<P>(&self, mut predicate: P) -> Option<Package<'_>>
    where
        P: FnMut(&str) -> bool,
    {
        self.tables.iter().find_map(|dependencies| {
            find_from_dependencies(dependencies, &mut predicate, self.manifest.rust_ident())
        })
    }
}

fn manifest_path() -> Result<PathBuf> {
    env::var_os(MANIFEST_DIR).ok_or(NotFoundManifestDir).map(PathBuf::from).map(
        |mut manifest_path| {
            manifest_path.push("Cargo.toml");
            manifest_path
        },
    )
}

fn find_from_dependencies<P>(table: &Table, mut predicate: P, convert: bool) -> Option<Package<'_>>
where
    P: FnMut(&str) -> bool,
{
    fn package<P>(value: &Value, predicate: P) -> Option<&str>
    where
        P: FnOnce(&str) -> bool,
    {
        value.as_table().and_then(|t| t.get("package")).and_then(Value::as_str).and_then(|s| {
            if predicate(s) {
                Some(s)
            } else {
                None
            }
        })
    }

    fn version(value: &Value) -> Option<&str> {
        value
            .as_str()
            .or_else(|| value.as_table().and_then(|t| t.get("version")).and_then(Value::as_str))
    }

    fn rust_ident(s: &str, convert: bool) -> Cow<'_, str> {
        if convert {
            Cow::Owned(s.replace("-", "_"))
        } else {
            Cow::Borrowed(s)
        }
    }

    table.iter().find_map(|(key, value)| {
        if predicate(key) {
            Some(Package {
                key,
                version: version(value),
                package: None,
                rust_ident: rust_ident(key, convert),
            })
        } else {
            package(value, &mut predicate).map(|package| Package {
                key,
                version: version(value),
                package: Some(package),
                rust_ident: rust_ident(key, convert),
            })
        }
    })
}
