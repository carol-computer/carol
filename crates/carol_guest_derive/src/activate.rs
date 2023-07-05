pub fn activate(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // NOTE: passthrough for now
    let input = syn::parse2::<syn::ItemFn>(input).expect("can only place #[activate] on an fn");

    quote::quote! { #input }
}
