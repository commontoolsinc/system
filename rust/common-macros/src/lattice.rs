use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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
