# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

Releases may yanked if there is a security bug, a soundness bug, or a regression.

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

- Update `toml` dependency to 1.

## [0.7.0] - 2026-02-07

- Remove `toml` from public dependencies to prevent `toml` updates from becoming breaking changes. ([#27](https://github.com/taiki-e/find-crate/pull/27), thanks @TechnoPorg)
  - Remove `Manifest::from_toml`. Use `text.parse()`/`Manifest::from_str(text)` or `Manifest::from_path(path)` instead.
  - Change the type of `Error::Toml` from `toml::de::Error` to `TomlError`.

- Make `Dependencies` enum `#[non_exhaustive]`.

- Implement `FromStr` for `Manifest`. ([#27](https://github.com/taiki-e/find-crate/pull/27), thanks @TechnoPorg)

- Add `Manifest::from_path`.

- Support [version-less manifests](https://github.com/rust-lang/cargo/pull/12786) in `crate_package`.

- Support target names that contain ".".

- Add `#[must_use]` to constructor and getters.

- Update `toml` dependency to 0.9.

  This increases the minimum supported Rust version (MSRV) to Rust 1.76.

- Enable [release immutability](https://docs.github.com/en/code-security/supply-chain-security/understanding-your-software-supply-chain/immutable-releases).

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

- Make `Manifest::dependencies` and `Package::{name, version}` fields public.

- Add support for `target.cfg.dependencies`.

- Add `Dependencies` enum to manage the kind of dependencies to be searched.

- Remove `Manifest::lock()` and `ManifestLock`.

- Remove some variant and field form `Error`.

- Remove `DEFAULT_DEPENDENCIES`.

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

[Unreleased]: https://github.com/taiki-e/find-crate/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/taiki-e/find-crate/compare/v0.6.3...v0.7.0
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
