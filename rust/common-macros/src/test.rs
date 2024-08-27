use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Attribute, Ident, ItemFn};

pub fn common_browser_integration_test(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = input;

    let test_name = sig.ident.clone();
    let test_name_string = test_name.to_string();

    let id = blake3::hash(test_name_string.as_bytes()).to_string();
    let inner_test_name = Ident::new(
        &format!("{test_name_string}_{}", &id.as_str()[0..6]),
        test_name.span(),
    );
    let inner_test_name_string = inner_test_name.to_string();

    sig.ident = inner_test_name;

    let tokio_test: Attribute = parse_quote! {
        #[tokio::test(flavor = "multi_thread")]
    };
    let wasm_bindgen_test: Attribute = parse_quote! {
        #[wasm_bindgen_test::wasm_bindgen_test]
    };
    let cfg_only_native: Attribute = parse_quote! {
        #[cfg(not(target_arch = "wasm32"))]
    };
    let cfg_only_browser_integration_test: Attribute = parse_quote! {
        #[cfg(all(common_browser_integration_test, target_arch = "wasm32", target_os = "unknown"))]
    };
    let cfg_feature_helpers: Attribute = parse_quote! {
        #[cfg(feature = "helpers")]
    };

    let common_tracing: Attribute = parse_quote! {
        #[common_macros::common_tracing]
    };

    quote!(
        #cfg_feature_helpers
        #cfg_only_native
        #tokio_test
        #common_tracing
        async fn #test_name () -> anyhow::Result<()> {
            use common_runtime::helpers::{VirtualEnvironment, start_runtime};
            use std::module_path;

            let VirtualEnvironment {
                runtime_port,
                builder_port,
                runtime_task,
                builder_task,
                ..
            } = start_runtime().await?;

            let status = std::process::Command::new("cargo")
                .env("COMMON_BUILDER_PORT", format!("{builder_port}"))
                .env("COMMON_RUNTIME_PORT", format!("{runtime_port}"))
                .env("COMMON_BROWSER_INTEGRATION_TEST", "true")
                .args([
                    "test",
                    "--target",
                    "wasm32-unknown-unknown",
                    #inner_test_name_string
                ])
                .status()?;

            assert!(status.success(), "The interior test failed");

            runtime_task.abort();
            builder_task.abort();

            Ok(())
        }


        #cfg_only_browser_integration_test
        #(#attrs)*
        #wasm_bindgen_test
        #vis #sig #block
    )
    .into()
}
