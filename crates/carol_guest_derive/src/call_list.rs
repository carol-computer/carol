use maud::{html, PreEscaped};
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_quote, parse_quote_spanned, spanned::Spanned, token, Arm, Expr, ExprMatch, LitStr, Path,
    ReturnType, Signature, Token,
};

use crate::activate::HttpMethod;
use std::collections::BTreeMap;

pub struct ActivationList {
    http_endpoints: BTreeMap<String, Methods>,
    methods: BTreeMap<String, Activation>,
}

struct Methods {
    map: BTreeMap<HttpMethod, Activation>,
    default_span: Span,
}

#[derive(Clone)]
pub struct HttpEndpoint {
    pub path: LitStr,
    pub method: HttpMethod,
}

#[derive(Clone)]
pub struct Activation {
    pub http_endpoint: Option<HttpEndpoint>,
    pub sig: syn::Signature,
    pub docs: Option<String>,
}

fn doc_params(signature: &Signature) -> Vec<(String, String)> {
    let mut params = vec![];
    for fn_arg in &signature.inputs {
        match fn_arg {
            syn::FnArg::Typed(fn_arg) => match fn_arg.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => {
                    let ident_str = pat_ident.ident.to_string();
                    if ident_str == "_cap" || ident_str == "cap" {
                        continue;
                    }
                    params.push((ident_str, fn_arg.ty.span().source_text().unwrap()));
                }
                _arg => { /* ignore */ }
            },
            syn::FnArg::Receiver(_) => { /*ignore*/ }
        }
    }
    params
}

impl ActivationList {
    pub fn new() -> Self {
        Self {
            http_endpoints: Default::default(),
            methods: Default::default(),
        }
    }
    pub fn add_call(&mut self, activation: Activation) {
        if let Some(http_endpoint) = &activation.http_endpoint {
            let http_methods = self
                .http_endpoints
                .entry(http_endpoint.path.value())
                .or_insert_with(|| Methods {
                    map: Default::default(),
                    default_span: http_endpoint.path.span(),
                });

            http_methods
                .map
                .insert(http_endpoint.method, activation.clone());
        }

        self.methods
            .insert(activation.sig.ident.to_string(), activation);
    }

    pub fn binary_api(&self) -> syn::ExprStruct {
        let activations = self
            .methods
            .iter()
            .map(|(method_name, endpoint)| {
                let method_name = method_name.to_string();
                parse_quote_spanned! { endpoint.sig.span() =>
                    carol_guest::bind::exports::carol::machine::guest::ActivationDescription {
                        name: #method_name.into(),
                    }
                }
            })
            .collect::<Vec<syn::ExprStruct>>();
        parse_quote! {
            carol_guest::bind::exports::carol::machine::guest::BinaryApi {
                activations: vec![ #(#activations),* ],
            }
        }
    }

    pub fn render(&self) -> String {
        html! {
            @for (path, endpoints) in &self.http_endpoints {
                @for (method, endpoint) in &endpoints.map {
                    @let desc_html = comrak::markdown_to_html(endpoint.docs.as_deref().unwrap_or(""), &comrak::ComrakOptions::default());
                    h2 { (method) " " (path) }
                    p { (PreEscaped(desc_html)) }
                    h3 { "Paramters" }
                    ol {
                        @for (name, ty) in &doc_params(&endpoint.sig) {
                            li { (name) ": " code { (ty) } }
                        }
                    }
                }
            }
        }.into_string()
    }

    pub fn to_match_arms(&self, carol_mod: &Ident) -> Vec<Arm> {
        let mut match_arms = vec![];
        let mut allowed: Vec<String> = vec![];

        for (path, endpoints) in &self.http_endpoints {
            let route_path = LitStr::new(path, endpoints.default_span);
            let mut inner_match_arms: Vec<Arm> = vec![];

            for (method, endpoint) in &endpoints.map {
                let method_name = endpoint.sig.ident.clone();
                let sig_span = method_name.span();

                let struct_name = Ident::new(
                    &heck::AsUpperCamelCase(&method_name.to_string()).to_string(),
                    endpoint.sig.ident.span(),
                );

                let struct_path: Path = parse_quote!(#carol_mod::#struct_name);

                let route_method_ident = match method {
                    HttpMethod::Post => format_ident!("Post"),
                    HttpMethod::Get => format_ident!("Get"),
                };

                allowed.push(method.to_string());

                let decode_code = match method {
                    HttpMethod::Post => {
                        quote_spanned! { sig_span => {
                            carol_guest::serde_json::from_slice::<#struct_path>(body).map_err(|e| format!("{:?}", e))
                        }}
                    }
                    HttpMethod::Get => {
                        quote_spanned! { sig_span => {
                            carol_guest::serde_urlencoded::from_str::<#struct_path>(__query).map_err(|e| e.to_string())
                        }}
                    }
                };

                let handle_output = match endpoint.sig.output.clone() {
                    ReturnType::Default => {
                        quote_spanned! { sig_span => http::Response {
                            headers: vec![],
                            status: 204,
                            body: vec![]
                        }}
                    }
                    ReturnType::Type(_, ty) => {
                        let bincode_decode_output_expect = format!(
                            "#[activate] bincode decoding the output of {} to type {}",
                            method_name,
                            ty.span()
                                .source_text()
                                .unwrap_or(ty.to_token_stream().to_string())
                        );
                        let json_encode_output_expect = format!(
                            "#[activate] JSON encoding the output of {} from type {}",
                            method_name,
                            ty.span()
                                .source_text()
                                .unwrap_or(ty.to_token_stream().to_string())
                        );

                        let encode_output = quote_spanned! { ty.span() => carol_guest::serde_json::to_vec_pretty(&__decoded_output).expect(#json_encode_output_expect)
                        };

                        if let syn::Type::Path(type_path) = &*ty {
                            let last = type_path
                                .path
                                .segments
                                .last()
                                .map(|segment| segment.ident.to_string());
                            if Some("Result".into()) == last {
                                // TODO: Do something different with results so we map Result Err to HTTP status codes
                            }
                        }

                        quote_spanned! { ty.span() =>  {
                            let (__decoded_output, _) : (#ty, _) = carol_guest::bincode::decode_from_slice(&output, carol_guest::bincode::config::standard()).expect(#bincode_decode_output_expect);
                            let json_encoded_output = #encode_output;
                            http::Response {
                                headers: vec![("Content-Type".into(), "application/json".as_bytes().to_vec())],
                                status: 200,
                                body: json_encoded_output
                            }
                        }}
                    }
                };

                let bincode_encode_error =
                    format!("#[activate] bincode encoding input to {}", method_name);

                let method_name_str = method_name.to_string();
                let arm_body = quote! {{
                    let method_struct = #decode_code;
                    let method_struct = match method_struct {
                        Ok(method_struct) => method_struct,
                        Err(e) => return http::Response {
                            headers: vec![],
                            body: e.as_bytes().to_vec(),
                            status: 400,
                        }
                    };


                    let binary_input: Vec<u8> = carol_guest::bincode::encode_to_vec(&method_struct, carol_guest::bincode::config::standard()).expect(#bincode_encode_error);
                    let output = match carol_guest::machines::Cap::self_activate(&__ctx, #method_name_str, &binary_input) {
                        Ok(output) => output,
                        Err(e) => return http::Response {
                            headers: vec![],
                            body: format!("HTTP handler failed to self-activate via {}, {}: {}", stringify!(#route_method_ident), #route_path, e).as_bytes().to_vec(),
                            status: 500,
                        }
                    };

                    #handle_output
                }};

                inner_match_arms.push(parse_quote_spanned!{sig_span => carol_guest::http::Method::#route_method_ident => #arm_body });
            }

            let allowed_str = allowed.join(", ");
            inner_match_arms.push(parse_quote_spanned! { endpoints.default_span => _ => {
                http::Response {
                    headers: vec![("Allow".into(), #allowed_str.as_bytes().to_vec())],
                    body: vec![],
                    status: 405,
                }
            }});

            let body = Box::new(Expr::Match(ExprMatch {
                attrs: vec![],
                match_token: Token![match](endpoints.default_span),
                expr: parse_quote_spanned!( endpoints.default_span => { __method } ),
                brace_token: token::Brace::default(),
                arms: inner_match_arms,
            }));

            match_arms.push(Arm {
                attrs: vec![],
                pat: parse_quote!( #route_path ),
                guard: None,
                fat_arrow_token: Token![=>](endpoints.default_span),
                body,
                comma: None,
            });
        }
        match_arms
    }
}
