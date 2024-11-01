use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn ct_tracing(item: TokenStream) -> TokenStream {
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
            ct_tracing::macro_impl::common_tracing_impl();
            #(#statements)*
        }
    )
    .into()
}
