// This file is @generated by find-crate-internal-codegen
// (gen_assert_impl function at tools/codegen/src/main.rs).
// It is not intended for manual editing.

#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#[allow(dead_code)]
fn assert_send<T: ?Sized + Send>() {}
#[allow(dead_code)]
fn assert_sync<T: ?Sized + Sync>() {}
#[allow(dead_code)]
fn assert_unpin<T: ?Sized + Unpin>() {}
#[allow(dead_code)]
fn assert_unwind_safe<T: ?Sized + std::panic::UnwindSafe>() {}
#[allow(dead_code)]
fn assert_ref_unwind_safe<T: ?Sized + std::panic::RefUnwindSafe>() {}
#[allow(unused_macros)]
macro_rules! assert_not_send {
    ($ty:ty) => {
        static_assertions::assert_not_impl_all!($ty : Send);
    };
}
#[allow(unused_macros)]
macro_rules! assert_not_sync {
    ($ty:ty) => {
        static_assertions::assert_not_impl_all!($ty : Sync);
    };
}
#[allow(unused_macros)]
macro_rules! assert_not_unpin {
    ($ty:ty) => {
        static_assertions::assert_not_impl_all!($ty : Unpin);
    };
}
#[allow(unused_macros)]
macro_rules! assert_not_unwind_safe {
    ($ty:ty) => {
        static_assertions::assert_not_impl_all!($ty : std::panic::UnwindSafe);
    };
}
#[allow(unused_macros)]
macro_rules! assert_not_ref_unwind_safe {
    ($ty:ty) => {
        static_assertions::assert_not_impl_all!($ty : std::panic::RefUnwindSafe);
    };
}
const _: fn() = || {
    assert_send::<crate::error::Error>();
    assert_sync::<crate::error::Error>();
    assert_unpin::<crate::error::Error>();
    assert_not_unwind_safe!(crate::error::Error);
    assert_not_ref_unwind_safe!(crate::error::Error);
    assert_send::<crate::Dependencies>();
    assert_sync::<crate::Dependencies>();
    assert_unpin::<crate::Dependencies>();
    assert_unwind_safe::<crate::Dependencies>();
    assert_ref_unwind_safe::<crate::Dependencies>();
    assert_send::<crate::Package>();
    assert_sync::<crate::Package>();
    assert_unpin::<crate::Package>();
    assert_unwind_safe::<crate::Package>();
    assert_ref_unwind_safe::<crate::Package>();
    assert_send::<crate::Manifest>();
    assert_sync::<crate::Manifest>();
    assert_unpin::<crate::Manifest>();
    assert_unwind_safe::<crate::Manifest>();
    assert_ref_unwind_safe::<crate::Manifest>();
};
