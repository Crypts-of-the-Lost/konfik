// top-level imports
use proc_macro::TokenStream; // <-- required for the proc-macro signature
use proc_macro2::Span; // for LitStr::new
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

#[proc_macro_derive(Config)]
pub fn derive_config(input: TokenStream) -> TokenStream {
    // parse the incoming proc_macro::TokenStream into a syn AST
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_meta = fields.iter().filter_map(|field| {
        let ident = field.ident.as_ref()?;
        let field_name = LitStr::new(&ident.to_string(), Span::call_site());

        Some(quote! {
            ::confik::FieldMeta {
                name: #field_name.to_string(),
                env_name: None,
                cli_name: None,
                skip: false,
            }
        })
    });

    let name_lit = LitStr::new(&name.to_string(), Span::call_site());

    // quote! produces a proc_macro2::TokenStream internally — that's fine
    let expanded = quote! {
        impl ::confik::ConfigMetadata for #name {
            fn config_metadata() -> ::confik::ConfigMeta {
                ::confik::ConfigMeta {
                    name: #name_lit.to_string(),
                    fields: vec![#(#field_meta),*],
                }
            }
        }

        impl ::confik::LoadConfig for #name {
            fn load() -> ::confik::Result<Self> {
                ::confik::ConfigLoader::new().load()
            }

            fn load_with(loader: &::confik::ConfigLoader) -> ::confik::Result<Self> {
                loader.load()
            }
        }
    };

    // convert proc_macro2::TokenStream -> proc_macro::TokenStream for the compiler
    TokenStream::from(expanded)
}
