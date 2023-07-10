use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{token, Token};

pub fn activate(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // NOTE: passthrough for now
    let input = syn::parse2::<syn::ItemFn>(input).expect("can only place #[activate] on an fn");

    quote::quote! { #input }
}

#[derive(Default)]
pub struct Opts {
    pub http: Option<HttpMethod>,
}

pub enum Opt {
    Http(HttpMethod),
}

pub enum HttpMethod {
    Post,
    Get,
}

impl Parse for Opts {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut opts = Opts::default();
        if input.peek(token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let fields = Punctuated::<Opt, Token![,]>::parse_terminated(&content)?;
            for field in fields.into_pairs() {
                match field.into_value() {
                    Opt::Http(s) => {
                        opts.http = Some(s);
                    }
                }
            }
        }

        Ok(opts)
    }
}

impl Parse for Opt {
    fn parse(input: ParseStream) -> Result<Self> {
        let l = input.lookahead1();

        if l.peek(kw::http) {
            input.parse::<kw::http>()?;

            if input.peek(token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let http_method = content.parse::<HttpMethod>()?;
                Ok(Opt::Http(http_method))
            } else {
                Err(Error::new(
                    input.span(),
                    "http must be followed by parentheses e.g. http(GET)",
                ))
            }
        } else {
            Err(l.error())
        }
    }
}

impl Parse for HttpMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = Span::call_site();
        let l = input.lookahead1();
        if l.peek(http_methods::GET) {
            input.parse::<http_methods::GET>()?;
            Ok(HttpMethod::Get)
        } else if l.peek(http_methods::POST) {
            input.parse::<http_methods::POST>()?;
            Ok(HttpMethod::Post)
        } else {
            return Err(Error::new(span, "Expecting either GET or POST"));
        }
    }
}

mod kw {
    syn::custom_keyword!(http);
}

mod http_methods {
    syn::custom_keyword!(GET);
    syn::custom_keyword!(POST);
}
