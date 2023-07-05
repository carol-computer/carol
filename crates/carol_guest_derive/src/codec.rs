use quote::quote;
use syn::{parse_macro_input, parse_quote};

pub fn codec(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input: syn::DeriveInput = parse_macro_input!(input);

    input.attrs.push(parse_quote!(#[derive(carol_guest::bincode::Decode, carol_guest::bincode::Encode, carol_guest::serde::Serialize, carol_guest::serde::Deserialize)]),);
    input
        .attrs
        .push(parse_quote!(#[serde(crate = "carol_guest::serde")]));
    input
        .attrs
        .push(parse_quote!(#[bincode(crate = "carol_guest::bincode")]));

    quote! {
        #input
    }
    .into()
}
