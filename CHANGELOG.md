# Unreleased

# 0.5.0 - 2019-09-29

* Made `Manifest::dependencies` and `Package::{name, varsion}` fields public.

* Added support for `target.cfg.dependencies`.

* Added `Dependencies` enum to manage the kind of dependencies to be searched.

* Removed `Manifest::lock()` and `ManifestLock`.

* Removed some variant and field form `Error`.

* Removed `DEFAULT_DEPENDENCIES`.

# 0.4.0 - 2019-06-16

* Transition to Rust 2018. With this change, the minimum required version will go up to Rust 1.31.

* Updated minimum `toml` version to 0.5.0.

# 0.3.0 - 2019-02-21

* Removed version dependent behavior.

* Improved documentation.

# 0.2.0 - 2019-02-13

* Supported Rust 1.15.

# 0.1.2 - 2019-02-13

* Implemented `PartialEq` and `Eq` for `Package`.

# 0.1.1 - 2019-02-13

* Improved documentation.

# 0.1.0 - 2019-02-13

Initial release
