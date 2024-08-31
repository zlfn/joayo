extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ToStatusCode, attributes(status_code))]
pub fn to_status_code(input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    impl_to_status_code(&item)
}

fn impl_to_status_code(item: &syn::DeriveInput) -> TokenStream {
    let attr = item.attrs.iter().filter(
        |a| a.path.segments.len() == 1 && a.path.segments[0].ident == "status_code"
    ).nth(0).expect("status_code required to derive ToStatusCode");

    let attr: proc_macro2::TokenStream = attr.parse_args().unwrap();

    let item = &item.ident;

    quote! {
        impl ToStatusCode for #item {
            fn to_status_code(&self) -> StatusCode {
                return StatusCode::#attr;
            }
        }
    }.into()
}

#[proc_macro_derive(FromSessionError)]
pub fn from_session_error(input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    impl_from_session_error(&item)
}

fn impl_from_session_error(item: &syn::DeriveInput) -> TokenStream {
    let item = &item.ident;

    quote! {
        impl crate::common::session::FromSessionError for #item {
            fn from_session_error(session_error: crate::common::session::SessionError) -> Self {
                match session_error {
                    crate::common::session::SessionError::SessionInvalid => Self::SessionInvalid,
                    crate::common::session::SessionError::InternalServerError => Self::InternalServerError,
                }
            }
        }
    }.into()
}
