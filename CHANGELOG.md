# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

- `find_crate` function is now returns `Result<Option<String>>` instead of `Result<Package>`. `Ok(None)` is equivalent to `Err(Error::NotFound)` in previous version.

- `Manifest::find` method is now returns `Option<String>` instead of `Option<Package>`. If you want to get `Package`, use `Manifest::find_package` method instead.

- Rename `Manifest::find2` method to `Manifest::find_package`.

- `Manifest::from_toml` function is now takes `&str` instead of `toml::Table`.

- Change `Error` from enum to struct.

- Change `Package::version` field from `String` to `&str`.

- Make `Dependencies` non-exhaustive.

- Add `Result` type alias.

- Remove `Package::is_original` method.

- Change closures in argument position from type parameter to impl trait.

- Documentation improvements.

## [0.6.3] - 2021-01-05

- Exclude unneeded files from crates.io.

## [0.6.2] - 2020-12-29

- Documentation improvements.

## [0.6.1] - 2020-09-07

- Documentation improvements.

## [0.6.0] - 2020-08-27

- [Add `Manifest::crate_package`.](https://github.com/taiki-e/find-crate/pull/12)

- Make `Error` non-exhaustive.

## [0.5.0] - 2019-09-29

- Make `Manifest::dependencies` and `Package::{name, varsion}` fields public.

- Add support for `target.cfg.dependencies`.

- Add `Dependencies` enum to manage the kind of dependencies to be searched.

- Remove `Manifest::lock` method and `ManifestLock`.

- Remove some variants and fields form `Error`.

- Remove `DEFAULT_DEPENDENCIES` constant.

## [0.4.0] - 2019-06-16

- Transition to Rust 2018. With this change, the minimum required version will go up to Rust 1.31.

- Update minimum `toml` version to 0.5.0.

## [0.3.0] - 2019-02-21

- Remove version dependent behavior.

- Documentation improvements.

## [0.2.0] - 2019-02-13

- Support Rust 1.15.

## [0.1.2] - 2019-02-13

- Implement `PartialEq` and `Eq` for `Package`.

## [0.1.1] - 2019-02-13

- Documentation improvements.

## [0.1.0] - 2019-02-13

Initial release

[Unreleased]: https://github.com/taiki-e/find-crate/compare/v0.6.3...HEAD
[0.6.3]: https://github.com/taiki-e/find-crate/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/taiki-e/find-crate/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/taiki-e/find-crate/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/taiki-e/find-crate/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/taiki-e/find-crate/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/taiki-e/find-crate/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/taiki-e/find-crate/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/taiki-e/find-crate/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/taiki-e/find-crate/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/taiki-e/find-crate/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/find-crate/releases/tag/v0.1.0
