use maud::{html, PreEscaped};
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_quote, parse_quote_spanned, spanned::Spanned, token, Arm, Expr, ExprMatch, LitStr, Path,
    ReturnType, Signature, Token,
};

use crate::activate::{Http, HttpMethod};
use std::collections::BTreeMap;

pub struct HttpCallList(BTreeMap<String, Endpoints>);

struct Endpoint {
    docs: Option<String>,
    sig: syn::Signature,
}

pub struct Endpoints {
    points: BTreeMap<HttpMethod, Endpoint>,
    default_span: Span,
}

pub struct Call {
    pub path: LitStr,
    pub http_opt: Http,
    pub sig: syn::Signature,
    pub docs: Option<String>,
}

fn html_doc_params(signature: &Signature) -> Vec<(String, String)> {
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

impl HttpCallList {
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn add_call(&mut self, call: Call) {
        let endpoints = self
            .0
            .entry(call.path.value())
            .or_insert_with(|| Endpoints {
                points: Default::default(),
                default_span: call.path.span(),
            });

        endpoints.points.insert(
            call.http_opt.method,
            Endpoint {
                docs: call.docs,
                sig: call.sig,
            },
        );
    }

    pub fn render(&self) -> String {
        html! {
            @for (path, endpoints) in &self.0 {
                @for (method, endpoint) in &endpoints.points {
                    @let desc_html = comrak::markdown_to_html(endpoint.docs.as_deref().unwrap_or(""), &comrak::ComrakOptions::default());
                    h2 { (method) " " (path) }
                    p { (PreEscaped(desc_html)) }
                    h3 { "Paramters" }
                    ol {
                        @for (name, ty) in &html_doc_params(&endpoint.sig) {
                            li { (name) ": " code { (ty) } }
                        }
                    }
                }
            }
        }.into_string()
    }

    pub fn to_match_arms(&self, carol_mod: &Ident, enum_name: &Ident) -> Vec<Arm> {
        let mut match_arms = vec![];
        let mut allowed: Vec<String> = vec![];

        for (path, endpoints) in &self.0 {
            let route_path = LitStr::new(path, endpoints.default_span);
            let mut inner_match_arms: Vec<Arm> = vec![];

            for (method, endpoint) in &endpoints.points {
                let method_name = endpoint.sig.ident.clone();
                let sig_span = method_name.span();

                let struct_name = Ident::new(
                    &heck::AsUpperCamelCase(&method_name.to_string()).to_string(),
                    endpoint.sig.ident.span(),
                );

                let struct_path: Path = parse_quote!(#carol_mod::#struct_name);
                let variant_path: Path = parse_quote!(#carol_mod::#enum_name::#struct_name);

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
                            ty.to_token_stream()
                        );
                        let json_encode_output_expect = format!(
                            "#[activate] JSON encoding the output of {} from type {}",
                            method_name,
                            ty.to_token_stream()
                        );

                        quote_spanned! { ty.span() =>  {
                            let (decoded_output, _) : (#ty, _) = carol_guest::bincode::decode_from_slice(&output, carol_guest::bincode::config::standard()).expect(#bincode_decode_output_expect);
                            let json_encoded_output = carol_guest::serde_json::to_vec_pretty(&decoded_output).expect(#json_encode_output_expect);
                            http::Response {
                                headers: vec![],
                                status: 200,
                                body: json_encoded_output
                            }
                        }}
                    }
                };

                let bincode_encode_error =
                    format!("#[activate] bincode encoding input to {}", method_name);

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
                    let method_variant = #variant_path(method_struct);

                    let binary_input: Vec<u8> = carol_guest::bincode::encode_to_vec(&method_variant, carol_guest::bincode::config::standard()).expect(#bincode_encode_error);
                    let output = match carol_guest::machines::Cap::self_activate(&__ctx, &binary_input) {
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
