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
    pub http: Option<Http>,
}

pub enum Opt {
    Http(Http),
}

pub struct Http {
    pub method: HttpMethod,
    pub path: Option<syn::LitStr>,
}

#[derive(PartialOrd, Ord, Debug, PartialEq, Eq, Clone, Copy)]
pub enum HttpMethod {
    Post,
    Get,
}

impl core::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Get => write!(f, "GET"),
        }
    }
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
                Ok(Opt::Http(content.parse::<Http>()?))
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

impl Parse for Http {
    fn parse(input: ParseStream) -> Result<Self> {
        let l = input.lookahead1();
        let method = if l.peek(http_methods::GET) {
            input.parse::<http_methods::GET>()?;
            HttpMethod::Get
        } else if l.peek(http_methods::POST) {
            input.parse::<http_methods::POST>()?;
            HttpMethod::Post
        } else {
            return Err(l.error());
        };
        let l = input.lookahead1();

        let path = if l.peek(syn::LitStr) {
            Some(input.parse::<syn::LitStr>()?)
        } else if input.is_empty() {
            None
        } else {
            return Err(l.error());
        };

        Ok(Http { method, path })
    }
}

mod kw {
    syn::custom_keyword!(http);
}

mod http_methods {
    syn::custom_keyword!(GET);
    syn::custom_keyword!(POST);
}
