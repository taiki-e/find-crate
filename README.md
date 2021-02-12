# find-crate

[![crates.io](https://img.shields.io/crates/v/find-crate?style=flat-square&logo=rust)](https://crates.io/crates/find-crate)
[![docs.rs](https://img.shields.io/badge/docs.rs-find--crate-blue?style=flat-square)](https://docs.rs/find-crate)
[![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)](#license)
[![rustc](https://img.shields.io/badge/rustc-1.31+-blue?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![build status](https://img.shields.io/github/workflow/status/taiki-e/find-crate/CI/master?style=flat-square&logo=github)](https://github.com/taiki-e/find-crate/actions)

Find the crate name from the current `Cargo.toml`.

When writing declarative macros, `$crate` representing the current crate is
very useful, but procedural macros do not have this. If you know the current
name of the crate you want to use, you can do the same thing as `$crate`.
This crate provides the features to make it easy.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
find-crate = "0.5"
```

*Compiler support: requires rustc 1.31+*

## Examples

[`find_crate`] function gets the crate name from the current `Cargo.toml`.

```rust
use find_crate::find_crate;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

fn import() -> TokenStream {
    let name = find_crate(|name| name == "foo").unwrap().unwrap();
    let name = Ident::new(&name, Span::call_site());
    // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    quote!(extern crate #name as _foo;)
}
```

As in this example, it is easy to handle cases where proc-macro is exported
from multiple crates.

```rust
use find_crate::find_crate;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

fn import() -> TokenStream {
    let name = find_crate(|name| name == "foo" || name == "foo-core").unwrap().unwrap();
    let name = Ident::new(&name, Span::call_site());
    // If your proc-macro crate is 2018 edition, use `quote!(use #name as _foo;)` instead.
    quote!(extern crate #name as _foo;)
}
```

Using [`Manifest`] to search for multiple crates. It is much more efficient
than using [`find_crate`] function for each crate.

```rust
use find_crate::Manifest;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

const CRATE_NAMES: &[&[&str]] = &[
    &["foo", "foo-core"],
    &["bar", "bar-util", "bar-core"],
    &["baz"],
];

fn imports() -> TokenStream {
    let mut tokens = TokenStream::new();
    let manifest = Manifest::new().unwrap();

    for names in CRATE_NAMES {
        let name = manifest.find(|name| names.iter().any(|x| name == *x)).unwrap();
        let name = Ident::new(&name, Span::call_site());
        let import_name = format_ident!("_{}", names[0]);
        // If your proc-macro crate is 2018 edition, use `quote!(use #name as #import_name;)` instead.
        tokens.extend(quote!(extern crate #name as #import_name;));
    }
    tokens
}
```

By default it will be searched from `dependencies` and `dev-dependencies`.
This behavior can be adjusted by changing the [`Manifest::dependencies`] field.

[`find_crate`] and [`Manifest::new`] functions read `Cargo.toml` in
[`CARGO_MANIFEST_DIR`] as manifest.

## Alternatives

If you write function-like procedural macros, [you can combine it with
declarative macros to support both crate renaming and macro
re-exporting.][rust-lang/futures-rs#2124]

This crate is intended to provide more powerful features such as support
for multiple crate names and versions. For general purposes,
[proc-macro-crate], which provides a simpler API, may be easier to use.

[`CARGO_MANIFEST_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
[proc-macro-crate]: https://github.com/bkchr/proc-macro-crate
[rust-lang/futures-rs#2124]: https://github.com/rust-lang/futures-rs/pull/2124
[`Manifest::new`]: https://docs.rs/find-crate/0.6/find_crate/struct.Manifest.html#method.new
[`Manifest`]: https://docs.rs/find-crate/0.6/find_crate/struct.Manifest.html
[`find_crate`]: https://docs.rs/find-crate/0.6/find_crate/fn.find_crate.html

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
