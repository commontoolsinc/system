#![warn(missing_docs)]

//! Macros for common crates.

use proc_macro::TokenStream;

mod lattice;
mod new_type;
mod tracing;

extern crate proc_macro;

#[cfg(feature = "tracing")]
#[proc_macro_attribute]
/// An attribute macro for decorating tests with an
/// initialized [tracing_subscriber::Subscriber].
/// Requires the `common_tracing` dependency.
///
/// Implementation defined in `common_tracing::implementation::common_tracing_impl`
pub fn common_tracing(_args: TokenStream, item: TokenStream) -> TokenStream {
    tracing::common_tracing(item)
}

/// Implements the `common_ifc::Lattice` trait.
#[proc_macro_derive(Lattice)]
pub fn derive_lattice(input: TokenStream) -> TokenStream {
    lattice::derive_lattice(input)
}

/// Creates trait implementations for new type structs.
///
/// ```
/// # use common_macros::NewType;
/// #[derive(NewType)]
/// struct Foo(String);
/// ```
///
/// There are several categories of traits that can be optionally
/// implemented:
///
/// * From
///   * `From<Inner>` for `Type`.
/// * Into
///   * `From<Type>` for `Inner`.
/// * Inner
///   * `Type::inner(&self) -> &Inner`
///   * `Type::inner_mut(&mut self) -> &mut Inner`
///   * `Type::into_inner(self) -> Inner`
/// * Constructor
///   * `Type::new(Inner) -> Type`
/// * Deref
///   * `Deref<Target = Inner>` for `Type`
///   * `DerefMut<Target = Inner>` for `Type`
///
/// Using the `#[new_type(skip)]` helper, you can opt out of specific
/// derive categories.
///
/// ```
/// # use common_macros::NewType;
/// /// Generate implementations for all supported traits,
/// /// except `Inner` and `Constructor`.
/// #[derive(NewType)]
/// #[new_type(skip(Inner, Constructor))]
/// struct Foo(String);
/// ```
///
/// Similarly, you can specify only the traits to implement
/// with `#[new_type(only)]`.
///
/// ```
/// # use common_macros::NewType;
/// /// Only implement `From` and `Into`.
/// #[derive(NewType)]
/// #[new_type(only(From, Into))]
/// struct Foo(String);
/// ```
#[proc_macro_derive(NewType, attributes(new_type))]
pub fn derive_new_type(input: TokenStream) -> TokenStream {
    new_type::derive_new_type(input)
}
