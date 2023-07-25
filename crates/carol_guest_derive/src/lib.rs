mod activate;
mod codec;
mod machine;
use proc_macro2::TokenStream;
mod html;

#[proc_macro_attribute]
pub fn machine(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    match attr.to_string().as_str() {
        "" => {
            let output = machine::machine(input);
            proc_macro::TokenStream::from(output)
        }
        invalid => panic!("“{}” is not a valid machine attribute", invalid),
    }
}

#[proc_macro_attribute]
pub fn codec(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    codec::codec(attr, input)
}

#[proc_macro_attribute]
pub fn activate(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    activate::activate(input).into()
}
