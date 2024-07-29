#![warn(missing_docs)]

//! Macros for common crates.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn};

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

/// Adds several methods and traits for "new type" structs.
///
/// Implements the following methods:
/// * `Type::inner(&self) -> &Inner`
/// * `Type::inner_mut(&mut self) -> &mut Inner`
/// * `Type::into_inner(self) -> Inner`
///
/// Implements the following traits:
/// * `impl Deref for Type`
/// * `impl DerefMut for Type`
/// * `impl From<Inner> for Type`
/// * `impl From<Type> for Inner`
#[proc_macro_derive(NewType)]
pub fn derive_new_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let Data::Struct(data) = input.data else {
        panic!("NewType only works for struct types.");
    };

    let Fields::Unnamed(fields) = data.fields else {
        panic!("NewType requires new type structs.");
    };

    if fields.unnamed.len() != 1 {
        panic!("Must be a new type with single inner type.");
    }

    let inner = fields.unnamed.first().unwrap().ty.clone();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Returns the inner type.
            pub fn inner(&self) -> &#inner {
                &self.0
            }

            /// Returns the inner type.
            pub fn inner_mut(&mut self) -> &mut #inner {
                &mut self.0
            }

            /// Returns the inner type.
            pub fn into_inner(self) -> #inner {
                self.0
            }
        }

        impl #impl_generics ::core::ops::Deref for #name #ty_generics #where_clause {
            type Target = #inner;

            fn deref(&self) -> &Self::Target {
                self.inner()
            }
        }

        impl #impl_generics ::core::ops::DerefMut for #name #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.inner_mut()
            }
        }

        impl #impl_generics ::core::convert::From<#inner> for #name #ty_generics #where_clause {
            fn from(value: #inner) -> Self {
                #name(value)
            }
        }
        impl #impl_generics From<#name #ty_generics> for #inner #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                value.into_inner()
            }
        }
    };

    expanded.into()
}

/// Implements the `common_ifc::Lattice` trait.
#[proc_macro_derive(Lattice)]
pub fn derive_lattice(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let Data::Enum(data) = input.data else {
        panic!("LabelType only works for enums.");
    };

    let mut variants = vec![];
    for variant in data.variants.iter() {
        if !matches!(variant.fields, Fields::Unit) {
            panic!("Only Unit variants are supported.");
        }
        let v_name = &variant.ident;
        variants.push(quote! { #name::#v_name });
    }
    let variants_count = variants.len();
    let Some(bottom) = variants.first() else {
        panic!("Requires at least one variant.");
    };
    let Some(top) = variants.last() else {
        panic!("Requires at least one variant.");
    };

    let pkg_name = std::env::var("CARGO_PKG_NAME").ok().unwrap_or_default();

    // Target trait path for consumers, as well as within `common-ifc`.
    let lattice_trait = if pkg_name == "common-ifc" {
        quote! { crate::Lattice }
    } else {
        quote! { common_ifc::Lattice }
    };

    let expanded = quote! {
        impl #lattice_trait for #name {
            fn top() -> Self {
                #top
            }

            fn bottom() -> Self {
                #bottom
            }

            fn iter() -> ::core::slice::Iter<'static, #name> {
                static VARIANTS: [#name; #variants_count] = [#(#variants),*];
                VARIANTS.iter()
            }
        }
    };

    expanded.into()
}
