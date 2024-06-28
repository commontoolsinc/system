#![warn(missing_docs)]
#![allow(unused)]

//! Macros for common crates.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

extern crate proc_macro;

#[cfg(feature = "tracing")]
#[proc_macro_attribute]
/// An attribute macro for decorating tests with an
/// initialized [tracing_subscriber::Subscriber].
/// Requires the `common_tracing` dependency.
///
/// Implementation defined in `common_tracing::implementation::common_tracing_impl`
pub fn common_tracing(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let ItemFn {
        sig,
        vis,
        block,
        attrs,
    } = input;
    let statements = block.stmts;

    quote!(
        #(#attrs)*
        #vis #sig {
            common_tracing::macro_impl::common_tracing_impl();
            #(#statements)*
        }
    )
    .into()
}
