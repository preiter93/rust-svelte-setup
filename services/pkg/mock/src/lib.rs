//! Procedural macros for generating mock implementations.
//!
//! # db_client
//!
//! Generates mock implementations for async DB client traits
//!
//! Use `#[cfg_attr(test, mock::db_client)]` before `#[async_trait]` to generate
//! the mock only during test compilation:
//!
//! ```ignore
//! use tonic::async_trait;
//!
//! #[cfg_attr(test, mock::db_client)]
//! #[async_trait]
//! pub trait DBClient: Send + Sync + 'static {
//!     async fn get_user(&self, id: Uuid) -> Result<User, DBError>;
//!     async fn insert_user(&self, id: Uuid, name: &str) -> Result<(), DBError>;
//! }
//!
//! // Generates:
//! // pub struct MockDBClient {
//! //     pub get_user: Mutex<Option<Result<User, DBError>>>,
//! //     pub get_user_call_count: AtomicUsize,
//! //     pub insert_user: Mutex<Option<Result<(), DBError>>>,
//! //     pub insert_user_call_count: AtomicUsize,
//! // }
//! // impl Default for MockDBClient { ... }
//! // #[async_trait] impl DBClient for MockDBClient { ... }
//! ```
//!
//! ## Checking Call Counts in Tests
//!
//! ```ignore
//! assert_eq!(mock.delete_session_calls(), 2);
//! ```

use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::{FnArg, ItemTrait, ReturnType, TraitItem, parse_macro_input};

/// Generates a mock implementation for an async trait.
#[proc_macro_attribute]
pub fn db_client(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    let trait_name = &input.ident;
    let mock_name = format_ident!("Mock{}", trait_name);
    let vis = &input.vis;

    let mut field_definitions = Vec::new();
    let mut default_fields = Vec::new();
    let mut impl_methods = Vec::new();
    let mut call_count_methods = Vec::new();

    for item in &input.items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let call_count_field = format_ident!("{}_call_count", method_name);
            let call_count_method = format_ident!("{}_calls", method_name);

            let return_type = match &method.sig.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => quote! { #ty },
            };

            field_definitions.push(quote! {
                pub #method_name: ::tokio::sync::Mutex<::std::option::Option<#return_type>>
            });

            field_definitions.push(quote! {
                pub #call_count_field: ::std::sync::atomic::AtomicUsize
            });

            default_fields.push(quote! {
                #method_name: ::tokio::sync::Mutex::new(::std::option::Option::None)
            });

            default_fields.push(quote! {
                #call_count_field: ::std::sync::atomic::AtomicUsize::new(0)
            });

            call_count_methods.push(quote! {
                pub fn #call_count_method(&self) -> usize {
                    self.#call_count_field.load(::std::sync::atomic::Ordering::SeqCst)
                }
            });

            let params: Vec<_> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let FnArg::Typed(pat_type) = arg {
                        let ty = &pat_type.ty;
                        let pat_str = pat_type.pat.to_token_stream().to_string();
                        let prefixed_name = format_ident!("_{}", pat_str);
                        Some(quote! { #prefixed_name: #ty })
                    } else {
                        None
                    }
                })
                .collect();

            impl_methods.push(quote! {
                async fn #method_name(&self, #(#params),*) -> #return_type {
                    self.#call_count_field.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
                    self.#method_name.lock().await.take().unwrap()
                }
            });
        }
    }

    let expanded = quote! {
        #input

        #vis struct #mock_name {
            #(#field_definitions),*
        }

        impl ::std::default::Default for #mock_name {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl #mock_name {
            #(#call_count_methods)*
        }

        #[::tonic::async_trait]
        impl #trait_name for #mock_name {
            #(#impl_methods)*
        }
    };

    TokenStream::from(expanded)
}
