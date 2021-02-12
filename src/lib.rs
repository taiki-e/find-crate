//! Find the crate name from the current `Cargo.toml`.
//!
//! When writing declarative macros, `$crate` representing the current crate is
//! very useful, but procedural macros do not have this. If you know the current
//! name of the crate you want to use, you can do the same thing as `$crate`.
//! This crate provides the features to make it easy.
//!
//! # Examples
//!
//! [`find_crate`] function gets the crate name from the current `Cargo.toml`.
//!
//! ```rust
//! use find_crate::find_crate;
//! use proc_macro2::{Ident, Span, TokenStream};
//! use quote::quote;
//!
//! fn import() -> TokenStream {
//!     let name = find_crate(|name| name == "foo").unwrap().unwrap();
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
//!     let name = find_crate(|name| name == "foo" || name == "foo-core").unwrap().unwrap();
//!     let name = Ident::new(&name, Span::call_site());
//!     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
//!     quote!(extern crate #name as _foo;)
//! }
//! ```
//!
//! Using [`Manifest`] to search for multiple crates. It is much more efficient
//! than using [`find_crate`] function for each crate.
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
//!     let mut tokens = TokenStream::new();
//!     let manifest = Manifest::new().unwrap();
//!
//!     for names in CRATE_NAMES {
//!         let name = manifest.find(|name| names.iter().any(|x| name == *x)).unwrap();
//!         let name = Ident::new(&name, Span::call_site());
//!         let import_name = format_ident!("_{}", names[0]);
//!         // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
//!         tokens.extend(quote!(extern crate #name as #import_name;));
//!     }
//!     tokens
//! }
//! ```
//!
//! By default it will be searched from `dependencies` and `dev-dependencies`.
//! This behavior can be adjusted by changing the [`Manifest::dependencies`] field.
//!
//! [`find_crate`] and [`Manifest::new`] functions read `Cargo.toml` in
//! [`CARGO_MANIFEST_DIR`] as manifest.
//!
//! # Alternatives
//!
//! If you write function-like procedural macros, [you can combine it with
//! declarative macros to support both crate renaming and macro
//! re-exporting.][rust-lang/futures-rs#2124]
//!
//! This crate is intended to provide more powerful features such as support
//! for multiple crate names and versions. For general purposes,
//! [proc-macro-crate], which provides a simpler API, may be easier to use.
//!
//! [`CARGO_MANIFEST_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
//! [proc-macro-crate]: https://github.com/bkchr/proc-macro-crate
//! [rust-lang/futures-rs#2124]: https://github.com/rust-lang/futures-rs/pull/2124

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![forbid(unsafe_code)]
#![warn(future_incompatible, rust_2018_idioms, unreachable_pub)]
// It cannot be included in the published code because these lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![warn(missing_debug_implementations, missing_docs)]
#![warn(clippy::all, clippy::default_trait_access)]

mod error;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use toml::value::{Table, Value};

use crate::error::ErrorKind;
pub use crate::error::{Error, Result};

/// The [`CARGO_MANIFEST_DIR`] environment variable.
///
/// [`CARGO_MANIFEST_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
const MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

/// Find the crate name from the current `Cargo.toml`.
///
/// This function reads `Cargo.toml` in [`CARGO_MANIFEST_DIR`] as manifest.
///
/// **Note:** This function needs to be used in the context of proc-macro.
///
/// # Return value
///
/// This function returns:
///
/// - `Ok(Some(name))` if the crate with the specified name found.
/// - `Ok(None)` if the crate with the specified name not found.
/// - `Err(err)` if an error occurred during reading or parsing the manifest file.
///
/// Returned crate name is always a valid rust identifier (`-` is replaced with `_`).
///
/// # Examples
///
/// ```rust
/// use find_crate::find_crate;
/// use proc_macro2::{Ident, Span, TokenStream};
/// use quote::quote;
///
/// fn import() -> TokenStream {
///     // Find the crate name from the current `Cargo.toml`.
///     let name: Option<String> = find_crate(|name| name == "foo" || name == "foo-core").unwrap();
///     // If you cannot find the crate, we recommend using the default (facade) crate name
///     // instead of panicking.
///     let name = name.as_deref().unwrap_or("foo");
///     let name = Ident::new(&name, Span::call_site());
///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
///     quote!(extern crate #name as _foo;)
/// }
/// ```
///
/// [`CARGO_MANIFEST_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
pub fn find_crate(predicate: impl FnMut(&str) -> bool) -> Result<Option<String>> {
    Ok(Manifest::new()?.find(predicate))
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

    // TODO: use real #[non_exhaustive] once MSRV bumped to 1.40.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Dependencies {
    fn as_slice(self) -> &'static [&'static str] {
        match self {
            Dependencies::Default => &["dependencies", "dev-dependencies"],
            Dependencies::Release => &["dependencies"],
            Dependencies::Dev => &["dev-dependencies"],
            Dependencies::Build => &["build-dependencies"],
            Dependencies::All => &["dependencies", "dev-dependencies", "build-dependencies"],
            Dependencies::__Nonexhaustive => unreachable!(),
        }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Dependencies::Default
    }
}

/// The package information. This has information on the current package name,
/// original package name, and specified version.
#[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/69952
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package<'a> {
    /// The key of this dependency in the manifest.
    key: &'a str,

    // The key or the value of 'package' key.
    // If this is `None`, the value of `key` field is the original name.
    package: Option<&'a str>,

    /// The current name of the package. This is always a valid rust identifier
    /// (`-` is replaced with `_`).
    pub name: String,

    /// The version requirement of the package. Returns `*` if no version
    /// requirement is specified.
    pub version: &'a str,
}

impl Package<'_> {
    /// Returns the original package name.
    pub fn original_name(&self) -> &str {
        self.package.unwrap_or(self.key)
    }
}

/// The manifest of cargo.
///
/// Note that this function needs to be used in the context of proc-macro.
#[derive(Debug, Clone)]
pub struct Manifest {
    manifest: Table,

    /// The kind of dependencies to be searched.
    pub dependencies: Dependencies,
}

impl Manifest {
    /// Creates a new `Manifest` from the current `Cargo.toml`.
    ///
    /// This function reads `Cargo.toml` in [`CARGO_MANIFEST_DIR`] as manifest.
    ///
    /// **Note:** This function needs to be used in the context of proc-macro.
    ///
    /// [`CARGO_MANIFEST_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    pub fn new() -> Result<Self> {
        Self::from_path(&manifest_path()?)
    }

    // TODO: Should we support custom manifest paths?
    //       And what should we do if the file is not found?
    //       (should we use `CARGO_MANIFEST_DIR`? Or should we return an error?)
    /// Creates a new `Manifest` from the specified manifest file.
    ///
    /// **Note:** This function needs to be used in the context of proc-macro.
    fn from_path(manifest_path: &Path) -> Result<Self> {
        let s = fs::read_to_string(manifest_path).map_err(Error::new)?;
        Self::from_toml(&s)
    }

    /// Creates a new `Manifest` from a toml text.
    ///
    /// **Note:** This function needs to be used in the context of proc-macro.
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(&s)
            .map_err(Error::new)
            .map(|manifest| Self { manifest, dependencies: Dependencies::default() })
    }

    /// Find the crate, and returns its crate name.
    ///
    /// The argument of the closure is the original name of the package.
    ///
    /// # Return value
    ///
    /// This function returns:
    ///
    /// - `Some(name)` if the crate with the specified name found.
    /// - `None` if the crate with the specified name not found.
    ///
    /// Returned crate name is always a valid rust identifier (`-` is replaced with `_`).
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
    ///     let name = manifest.find(|name| name == "foo" || name == "foo-core").unwrap();
    ///     let name = Ident::new(&name, Span::call_site());
    ///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    ///     quote!(extern crate #name as _foo;)
    /// }
    /// ```
    #[inline]
    pub fn find(&self, mut predicate: impl FnMut(&str) -> bool) -> Option<String> {
        self.find_package(|name, _| predicate(name)).map(|package| package.name)
    }

    /// Find the crate, and returns its package information.
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
    ///         .find_package(|name, req| name == "foo" && check_version(req, &version))
    ///         .unwrap()
    ///         .name;
    ///     let name = Ident::new(&name, Span::call_site());
    ///     // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    ///     quote!(extern crate #name as _foo;)
    /// }
    /// ```
    #[inline]
    pub fn find_package(&self, predicate: impl FnMut(&str, &str) -> bool) -> Option<Package<'_>> {
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
    /// fn current_crate_name() -> TokenStream {
    ///     let manifest = Manifest::new().unwrap();
    ///     let current_crate_package = manifest.crate_package().unwrap();
    ///     let name = Ident::new(&current_crate_package.name, Span::call_site());
    ///     quote!(#name)
    /// }
    /// ```
    pub fn crate_package(&self) -> Result<Package<'_>> {
        let package_section = self
            .manifest
            .get("package")
            .ok_or_else(|| Error::invalid_manifest("[package] section is missing"))?;

        let package_key_value = package_section
            .get("name")
            .ok_or_else(|| Error::invalid_manifest("[package] section is missing `name` field"))?;

        let package_key = package_key_value.as_str().ok_or_else(|| {
            Error::invalid_manifest("`name` field in [package] section is not a string")
        })?;

        let package_version_value = package_section.get("version").ok_or_else(|| {
            Error::invalid_manifest("[package] section is missing `version` field")
        })?;

        let package_version = package_version_value.as_str().ok_or_else(|| {
            Error::invalid_manifest("`version` field in [package] section is not a string")
        })?;

        let package = Package {
            key: package_key,
            package: None,
            name: package_key.replace("-", "_"),
            version: package_version,
        };

        Ok(package)
    }
}

fn manifest_path() -> Result<PathBuf> {
    let mut path: PathBuf = env::var_os(MANIFEST_DIR).ok_or(ErrorKind::NotFoundManifestDir)?.into();
    path.push("Cargo.toml");
    Ok(path)
}

fn find(
    manifest: &Table,
    dependencies: Dependencies,
    mut predicate: impl FnMut(&str, &str) -> bool,
) -> Option<Package<'_>> {
    fn find_inner<'a>(
        table: &'a Table,
        dependencies: &str,
        predicate: impl FnMut(&str, &str) -> bool,
    ) -> Option<Package<'a>> {
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

fn find_from_dependencies(
    table: &Table,
    mut predicate: impl FnMut(&str, &str) -> bool,
) -> Option<Package<'_>> {
    fn package<'a>(
        value: &'a Value,
        version: &str,
        predicate: impl FnOnce(&str, &str) -> bool,
    ) -> Option<&'a str> {
        value
            .as_table()?
            .get("package")?
            .as_str()
            .and_then(|s| if predicate(s, version) { Some(s) } else { None })
    }

    fn version(value: &Value) -> Option<&str> {
        value.as_str().or_else(|| value.as_table()?.get("version")?.as_str())
    }

    table.iter().find_map(|(key, value)| {
        let version = version(value).unwrap_or("*");
        let package = package(value, version, &mut predicate);
        if package.is_some() || predicate(key, version) {
            Some(Package { key, name: key.replace("-", "_"), version, package })
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_impl_all as assert_impl;

    use crate::*;

    assert_impl!(Manifest: Send);
    assert_impl!(Manifest: Sync);
    assert_impl!(Manifest: Unpin);

    assert_impl!(Package<'_>: Send);
    assert_impl!(Package<'_>: Sync);
    assert_impl!(Package<'_>: Unpin);

    assert_impl!(Dependencies: Send);
    assert_impl!(Dependencies: Sync);
    assert_impl!(Dependencies: Unpin);

    assert_impl!(Error: Send);
    assert_impl!(Error: Sync);
    assert_impl!(Error: Unpin);
}
