use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Generics, Ident, ImplGenerics, Type,
    TypeGenerics, WhereClause,
};

#[derive(PartialEq, Clone, Default, Debug, FromMeta)]
#[darling(default)]
struct Features {
    #[darling(rename = "From")]
    from: bool,
    #[darling(rename = "Into")]
    into: bool,
    #[darling(rename = "Inner")]
    inner: bool,
    #[darling(rename = "Constructor")]
    constructor: bool,
    #[darling(rename = "Deref")]
    deref: bool,
}

impl Features {
    pub fn invert(mut self) -> Self {
        self.from = !self.from;
        self.into = !self.into;
        self.inner = !self.inner;
        self.constructor = !self.constructor;
        self.deref = !self.deref;
        self
    }
}

#[derive(Default, PartialEq, Debug, FromDeriveInput)]
#[darling(default, attributes(new_type), supports(struct_newtype))]
struct NewTypeOpts {
    skip: Option<Features>,
    only: Option<Features>,
}

impl NewTypeOpts {
    /// Returns the list of features to implement, normalized
    /// to an `only` context.
    ///
    /// `only` takes precedence over `skip` if both defined.  
    pub fn features(&self) -> Features {
        match (self.skip.clone(), self.only.clone()) {
            (Some(_), Some(only)) => only,
            (None, Some(only)) => only,
            (Some(skip), None) => skip.invert(),
            (None, None) => Features::default().invert(),
        }
    }
}

struct ParsedInput {
    name: Ident,
    inner: Type,
    generics: Generics,
}

impl ParsedInput {
    fn expand(
        &self,
    ) -> (
        &Ident,
        &Type,
        ImplGenerics,
        TypeGenerics,
        Option<&WhereClause>,
    ) {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        (
            &self.name,
            &self.inner,
            impl_generics,
            ty_generics,
            where_clause,
        )
    }
}

impl From<(Ident, Type, Generics)> for ParsedInput {
    fn from(value: (Ident, Type, Generics)) -> Self {
        ParsedInput {
            name: value.0,
            inner: value.1,
            generics: value.2,
        }
    }
}

pub fn derive_new_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = NewTypeOpts::from_derive_input(&input).unwrap();
    let parsed = parse_new_type(input);

    let mut impls: Vec<TokenStream> = vec![];
    let features = opts.features();

    if features.from {
        impls.push(derive_from(&parsed));
    }
    if features.into {
        impls.push(derive_into(&parsed));
    }
    if features.constructor {
        impls.push(derive_constructor(&parsed));
    }
    if features.deref {
        impls.push(derive_deref(&parsed));
    }
    if features.inner {
        impls.push(derive_inner(&parsed));
    }

    impls.into_iter().collect()
}

/// Parses a [`TokenStream`], validates the new type,
/// and returns a [`ParsedInput`].
fn parse_new_type(input: DeriveInput) -> ParsedInput {
    let name = input.ident;
    let generics = input.generics;

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

    (name, inner, generics).into()
}

fn derive_from(input: &ParsedInput) -> TokenStream {
    let (name, inner, impl_generics, ty_generics, where_clause) = input.expand();
    quote! {
        impl #impl_generics ::core::convert::From<#inner> for #name #ty_generics #where_clause {
            fn from(value: #inner) -> Self {
                #name(value)
            }
        }
    }
    .into()
}

fn derive_into(input: &ParsedInput) -> TokenStream {
    let (name, inner, impl_generics, ty_generics, where_clause) = input.expand();
    quote! {
        impl #impl_generics ::core::convert::From<#name #ty_generics> for #inner #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                value.0
            }
        }
    }
    .into()
}

fn derive_inner(input: &ParsedInput) -> TokenStream {
    let (name, inner, impl_generics, ty_generics, where_clause) = input.expand();
    quote! {
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
    }
    .into()
}

fn derive_deref(input: &ParsedInput) -> TokenStream {
    let (name, inner, impl_generics, ty_generics, where_clause) = input.expand();
    quote! {
        impl #impl_generics ::core::ops::Deref for #name #ty_generics #where_clause {
            type Target = #inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl #impl_generics ::core::ops::DerefMut for #name #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    }
    .into()
}

fn derive_constructor(input: &ParsedInput) -> TokenStream {
    let (name, inner, impl_generics, ty_generics, where_clause) = input.expand();
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Creates a new [#name] from a [#inner].
            pub fn new(inner: #inner) -> Self {
                Self(inner)
            }
        }
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result<T> = std::result::Result<T, String>;

    fn parse(attrs: &str) -> Result<NewTypeOpts> {
        let def = format!(
            r#"
        #[derive(NewType)]
        {}
        struct Foo(u32);"#,
            attrs
        );
        FromDeriveInput::from_derive_input(&syn::parse_str(&def).map_err(|e| format!("{:#?}", e))?)
            .map_err(|e| format!("Could not parse {}: {}", attrs, e))
    }

    #[test]
    fn it_parses_attributes() -> Result<()> {
        // Redundant "empty" attributes
        for def in ["", "#[new_type]", "#[new_type()]"] {
            assert_eq!(
                parse(def)?,
                NewTypeOpts {
                    skip: None,
                    only: None,
                },
            );
        }

        // Parses `only`
        assert_eq!(
            parse("#[new_type(only(From, Into))]")?,
            NewTypeOpts {
                skip: None,
                only: Some(Features {
                    from: true,
                    into: true,
                    ..Default::default()
                }),
            }
        );

        // Parses `skip`
        assert_eq!(
            parse("#[new_type(skip(Constructor))]")?,
            NewTypeOpts {
                skip: Some(Features {
                    constructor: true,
                    ..Default::default()
                }),
                only: None,
            }
        );

        // Parseable, but derive macro will reject
        // if both `only` and `skip` provided.
        assert_eq!(
            parse("#[new_type(only(From, Into), skip(Constructor))]")?,
            NewTypeOpts {
                skip: Some(Features {
                    constructor: true,
                    ..Default::default()
                }),
                only: Some(Features {
                    from: true,
                    into: true,
                    ..Default::default()
                }),
            },
            "Defaults to `only`."
        );

        Ok(())
    }
}
