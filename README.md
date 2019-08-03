# find-crate

[![Build Status][azure-badge]][azure-url]
[![Crates.io][crates-version-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![License][crates-license-badge]][crates-url]
[![Minimum supported Rust version][rustc-badge]][rustc-url]

[azure-badge]: https://dev.azure.com/taiki-e/taiki-e/_apis/build/status/taiki-e.find-crate?branchName=master
[azure-url]: https://dev.azure.com/taiki-e/taiki-e/_build/latest?definitionId=10&branchName=master
[crates-version-badge]: https://img.shields.io/crates/v/find-crate.svg
[crates-license-badge]: https://img.shields.io/crates/l/find-crate.svg
[crates-badge]: https://img.shields.io/crates/v/find-crate.svg
[crates-url]: https://crates.io/crates/find-crate/
[docs-badge]: https://docs.rs/find-crate/badge.svg
[docs-url]: https://docs.rs/find-crate/
[rustc-badge]: https://img.shields.io/badge/rustc-1.31+-lightgray.svg
[rustc-url]: https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html

Find the crate name from the current `Cargo.toml` (`$crate` for proc-macro).

When writing declarative macros, `$crate` representing the current crate is very useful, but procedural macros do not have this. If you know the current name of the crate you want to use, you can do the same thing as `$crate`. This crate provides the features to make it easy.

[Documentation](https://docs.rs/find-crate/)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
find-crate = "0.4"
```

The current find-crate requires Rust 1.31 or later.

## Examples

`find_crate()` gets the crate name from the current `Cargo.toml`.

```rust
use find_crate::find_crate;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

fn import() -> TokenStream {
    let name = find_crate(|s| s == "foo").unwrap();
    let name = Ident::new(&name, Span::call_site());
    // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    quote!(extern crate #name as _foo;)
}
```

As in this example, it is easy to handle cases where proc-macro is exported from multiple crates.

```rust
use find_crate::find_crate;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

fn import() -> TokenStream {
    let name = find_crate(|s| s == "foo" || s == "foo-core").unwrap();
    let name = Ident::new(&name, Span::call_site());
    // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    quote!(extern crate #name as _foo;)
}
```

Search for multiple crates. It is much more efficient than using `find_crate()` for each crate.

```rust
use find_crate::Manifest;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

const CRATE_NAMES: &[&[&str]] = &[
    &["foo", "foo-core"],
    &["bar", "bar-util", "bar-core"],
    &["baz"],
];

fn imports() -> TokenStream {
    let mut tts = TokenStream::new();
    let manifest = Manifest::new().unwrap();
    let manifest = manifest.lock();

    for names in CRATE_NAMES {
        let name = manifest.find_name(|s| names.iter().any(|x| s == *x)).unwrap();
        let name = Ident::new(&name, Span::call_site());
        let import_name = Ident::new(&format!("_{}", names[0]), Span::call_site());
        // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
        tts.extend(quote!(extern crate #name as #import_name;));
    }
    tts
}
```

By default it will be searched from `dependencies`, `dev-dependencies` and `build-dependencies`.
Also, `find_crate()` and `Manifest::new()` read `Cargo.toml` in `CARGO_MANIFEST_DIR` as manifest.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
