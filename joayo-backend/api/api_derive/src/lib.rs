extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ToStatusCode, attributes(status_code))]
pub fn to_status_code(input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    impl_to_status_code_macro(&item)
}

fn impl_to_status_code_macro(item: &syn::DeriveInput) -> TokenStream {
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

