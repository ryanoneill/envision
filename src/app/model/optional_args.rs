//! Sealed marker trait gating the no-args shortcut on `RuntimeBuilder::build()`.
//!
//! `OptionalArgs` is implemented only for `()`. When `App::Args = ()`, consumers
//! can call `.build()` without `.with_args(...)`. For any other Args type, the
//! compiler requires `.with_args` to be called.

mod sealed {
    /// Internal supertrait — not part of envision's public surface.
    pub trait Sealed {
        fn default_optional_args() -> Self;
    }

    impl Sealed for () {
        fn default_optional_args() -> Self {}
    }
}

/// Marker trait gating the `Args = ()` shortcut on `RuntimeBuilder::build()`.
///
/// Sealed: consumers cannot extend it. Only `()` implements `OptionalArgs`.
/// Any consumer who wants the no-`with_args` shortcut on `RuntimeBuilder::build()`
/// must declare `type Args = ();` on their `App` impl.
///
/// # Sealed invariant
///
/// Only `()` implements `OptionalArgs`. Custom types cannot opt in:
///
/// ```compile_fail
/// use envision::OptionalArgs;
/// struct MyArgs;
/// impl OptionalArgs for MyArgs {}  // Compile error: trait `Sealed` is private
/// ```
pub trait OptionalArgs: sealed::Sealed {}

impl OptionalArgs for () {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_implements_optional_args() {
        fn assert_impl<T: OptionalArgs>() {}
        assert_impl::<()>();
    }
}
