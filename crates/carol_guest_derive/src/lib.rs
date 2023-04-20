use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse2, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Brace},
    Arm, Expr, ExprMatch, ExprMethodCall, ExprPath, Field, FieldPat, Fields, FieldsNamed,
    FieldsUnnamed, Generics, ImplItem, ItemEnum, ItemStruct, Pat, PatStruct, PatTuple,
    PatTupleStruct, Path, PathSegment, ReturnType, Token, Type, TypePath, Variant, VisPublic,
    Visibility,
};

#[proc_macro_attribute]
pub fn carol(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    match attr.to_string().as_str() {
        "" => {
            let output = carol_inner(input);
            proc_macro::TokenStream::from(output)
        }
        invalid => panic!("“{}” is not a valid carol attribute", invalid),
    }
}

#[proc_macro_attribute]
pub fn activate(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    match attr.to_string().as_str() {
        "" => activate_inner(input).into(),
        invalid => panic!("“{}” is not a valid activate attribute", invalid),
    }
}

fn activate_inner(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // NOTE: passthrough for now
    let input = parse2::<syn::ImplItem>(input).unwrap();
    input.to_token_stream()
}

fn carol_inner(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let input = parse2::<syn::ItemImpl>(input).unwrap();
    let enum_name = format_ident!("Activate");
    let carol_mod = format_ident!("carol_activate");
    let mut call_interface_enum = ItemEnum {
        attrs: vec![
            parse_quote!(#[derive(carol_guest::bincode::Decode, carol_guest::bincode::Encode, carol_guest::serde::Serialize, carol_guest::serde::Deserialize, Debug, Clone)]),
            parse_quote!(#[serde(crate = "carol_guest::serde")]),
            parse_quote!(#[bincode(crate = "carol_guest::bincode")]),
        ],
        vis: syn::Visibility::Public(VisPublic {
            pub_token: Token![pub](Span::call_site()),
        }),
        enum_token: Token![enum](Span::call_site()),
        ident: enum_name.clone(),
        generics: Generics::default(),
        brace_token: Brace::default(),
        variants: Punctuated::default(),
    };

    let mut match_arms: Vec<Arm> = vec![];
    let mut json_match_arms: Vec<Arm> = vec![];
    let mut method_structs = vec![];

    for item in input.items.clone() {
        if let ImplItem::Method(method) = item {
            if method
                .attrs
                .iter()
                .find(|attr| attr.path.to_token_stream().to_string() == "activate")
                .is_none()
            {
                continue;
            }
            let method_name = method.sig.ident.clone();
            let sig_span = method_name.span();
            let struct_name = Ident::new(
                &heck::AsUpperCamelCase(&method_name.to_string()).to_string(),
                method.sig.ident.span(),
            );
            let struct_path = Path {
                leading_colon: None,
                segments: Punctuated::from_iter(vec![PathSegment::from(struct_name.clone())]),
            };

            let mut struct_fields = FieldsNamed {
                brace_token: Brace::default(),
                named: Punctuated::default(),
            };

            let mut match_fields = Punctuated::default();
            for fn_arg in &method.sig.inputs {
                let span = fn_arg.span();
                let (field, match_field) = match fn_arg {
                    syn::FnArg::Receiver(_) => continue,
                    syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => (
                            Field {
                                attrs: vec![],
                                vis: syn::Visibility::Public(VisPublic {
                                    pub_token: Token![pub](span),
                                }),
                                ident: Some(pat_ident.ident.clone()),
                                colon_token: Some(pat_type.colon_token),
                                ty: *pat_type.ty.clone(),
                            },
                            FieldPat {
                                member: syn::Member::Named(pat_ident.ident),
                                colon_token: None,
                                pat: pat_type.pat.clone(),
                                attrs: vec![],
                            },
                        ),
                        _ => panic!("only take ident fn args"),
                    },
                };

                struct_fields.named.push(field);
                match_fields.push(match_field);
            }

            let struct_def = ItemStruct {
                attrs: vec![
                    parse_quote!(#[derive(carol_guest::bincode::Decode, carol_guest::bincode::Encode, carol_guest::serde::Serialize, carol_guest::serde::Deserialize, Debug, Clone)]),
                    parse_quote!(#[serde(crate = "carol_guest::serde")]),
                    parse_quote!(#[bincode(crate = "carol_guest::bincode")]),
                ],
                vis: syn::Visibility::Public(VisPublic {
                    pub_token: Token![pub](sig_span),
                }),
                struct_token: Token![struct](sig_span),
                ident: struct_name.clone(),
                generics: Generics::default(),
                fields: Fields::Named(struct_fields.clone()),
                semi_token: None,
            };

            method_structs.push(struct_def);

            let variant = Variant {
                attrs: Default::default(),
                ident: struct_name.clone(),
                discriminant: None,
                fields: Fields::Unnamed(FieldsUnnamed {
                    paren_token: token::Paren::default(),
                    unnamed: {
                        Punctuated::from_iter(vec![
                            (Field {
                                attrs: vec![],
                                vis: Visibility::Inherited,
                                ident: None,
                                colon_token: None,
                                ty: Type::Path(TypePath {
                                    qself: None,
                                    path: struct_path.clone(),
                                }),
                            }),
                        ])
                    },
                }),
            };

            let variant_path: Path = parse_quote!(#carol_mod::#enum_name::#struct_name);
            // let variant_path: Path = {
            //     let mut puncuated = Punctuated::new();
            //     puncuated.push(PathSegment::from(carol_mod.clone()));
            //     puncuated.push(PathSegment::from(enum_name.clone()));
            //     puncuated.push(PathSegment::from(variant.ident.clone()));
            //     Path {
            //         leading_colon: None,
            //         segments: puncuated,
            //     }
            // }

            let activate_call = ExprMethodCall {
                attrs: vec![],
                receiver: Box::new(Expr::Verbatim(quote! { machine })),
                dot_token: Token![.](Span::call_site()),
                method: method.sig.ident.clone(),
                turbofish: None,
                paren_token: token::Paren::default(),
                args: {
                    let mut punctuated = Punctuated::new();
                    for field in &struct_fields.named {
                        punctuated.push(Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path::from(PathSegment::from(field.ident.clone().unwrap())),
                        }));
                    }
                    punctuated
                },
            };

            let encode_output_expect = format!("Failed to encode output of {}", method_name);
            let encode_output_call = quote_spanned! { sig_span  => carol_guest::bincode::encode_to_vec(#activate_call, carol_guest::bincode::config::standard()).expect(#encode_output_expect) };

            match_arms.push(Arm {
                attrs: vec![],
                pat: Pat::TupleStruct(PatTupleStruct {
                    attrs: vec![],
                    path: variant_path.clone(),
                    pat: PatTuple {
                        attrs: vec![],
                        paren_token: token::Paren::default(),
                        elems: Punctuated::from_iter(vec![Pat::Struct(PatStruct {
                            attrs: vec![],
                            path: Path {
                                leading_colon: None,
                                segments: Punctuated::from_iter(vec![
                                    PathSegment::from(carol_mod.clone()),
                                    PathSegment::from(struct_name.clone()),
                                ]),
                            },
                            brace_token: token::Brace::default(),
                            fields: match_fields,
                            dot2_token: None,
                        })]),
                    },
                }),
                guard: None,
                fat_arrow_token: Token![=>](sig_span),
                body: Box::new(Expr::Verbatim(encode_output_call)),
                comma: None,
            });

            let route = format!("/activate/{}", method.sig.ident.to_string());

            let handle_output = match method.sig.output {
                ReturnType::Default => quote_spanned! { method.sig.span() => http::Response {
                    headers: vec![],
                    status: 204,
                    body: vec![]
                }},
                ReturnType::Type(_, ty) => {
                    let bincode_decode_output_expect = format!(
                        "#[carol] bincode decoding the output of {} to type {}",
                        method_name,
                        ty.to_token_stream().to_string()
                    );
                    let json_encode_output_expect = format!(
                        "#[carol]] JSON encoding the output of {} from type {}",
                        method_name,
                        ty.to_token_stream().to_string()
                    );
                    quote_spanned! { ty.span() =>  {
                        let (decoded_output, _) : (#ty, _) = bincode::decode_from_slice(&output, bincode::config::standard()).expect(#bincode_decode_output_expect);
                        let json_encoded_output = serde_json::to_vec_pretty(&decoded_output).expect(#json_encode_output_expect);
                        http::Response {
                            headers: vec![],
                            status: 200,
                            body: json_encoded_output
                        }
                    }}
                }
            };

            let pat = parse_quote! { #route };
            let struct_path: Path = parse_quote!(#carol_mod::#struct_path);
            let json_decode_error = format!(
                "#[carol] decoding JSON to {}",
                struct_path.to_token_stream().to_string().replace(" ", "")
            );
            let bincode_encode_error =
                format!("#[carol] bincode encoding input to {}", method_name);
            json_match_arms.push(Arm {
                attrs: vec![],
                pat,
                fat_arrow_token: Token![=>](sig_span),
                body: Box::new(Expr::Verbatim(quote_spanned! { sig_span => {
                        use carol_guest::{bincode, serde_json, machines};
                        let method_struct = carol_guest::serde_json::from_slice::<#struct_path>(body).expect(#json_decode_error);
                        let method_variant = #variant_path(method_struct);
                        let binary_input: Vec<u8> = carol_guest::bincode::encode_to_vec(&method_variant, carol_guest::bincode::config::standard())
                                              .expect(#bincode_encode_error);
                        let output = match machines::self_activate(&binary_input) {
                            Ok(output) => output,
                            Err(e) => return http::Response {
                                headers: vec![],
                                body: format!("HTTP handler failed to self-activate via {}: {}", #route, e).as_bytes().to_vec(),
                                status: 500,
                            }
                        };

                        #handle_output
                }})),
                comma: None,
                guard: None
            });

            call_interface_enum.variants.push(variant);
        }
    }

    let match_stmt = Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: Token![match](Span::call_site()),
        expr: Box::new(Expr::Verbatim(quote! { method })),
        brace_token: token::Brace::default(),
        arms: match_arms,
    });

    json_match_arms.push(parse_quote! { _ => {
        return http::Response {
            headers: vec![],
            body: vec![],
            status: 404
        }
    }});

    let json_match_stmt = Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: Token![match](Span::call_site()),
        expr: Box::new(Expr::Verbatim(quote! { path })),
        brace_token: token::Brace::default(),
        arms: json_match_arms,
    });

    let self_ty = input.self_ty.clone();
    let params_decode_expect = format!(
        "#[carol] bincode decoding parameters as {}",
        self_ty.to_token_stream().to_string().replace(" ", "")
    );
    let enum_path: Path = parse_quote!(#carol_mod::#enum_name);
    let input_decode_expect = format!(
        "#[carol] bincode decoding input as {}",
        enum_path.to_token_stream().to_string().replace(" ", "")
    );

    let output = quote! {

        pub mod #carol_mod {
            use super::*;
            #(#method_structs)*

            #call_interface_enum
        }

        #[cfg(not(test))]
        carol_guest::set_machine!(#self_ty);

        #input
        mod __machine_impl {
            use super::*;

            #[cfg(target_arch = "wasm32")]
            fn set_up_panic_hook() {
                let original_hook = std::panic::take_hook();
                std::panic::set_hook(Box::new(move |panic_info| {
                    carol_guest::log::set_panic_message(&panic_info.to_string());
                    (original_hook)(panic_info)
                }));
            }

            use carol_guest::{http, bincode};
            impl carol_guest::machine::Machine for #self_ty {
                fn activate(__params: Vec<u8>, __input: Vec<u8>) -> Vec<u8> {
                    use carol_guest::machines;
                    #[cfg(target_arch = "wasm32")]
                    set_up_panic_hook();
                    let (machine, _) = bincode::decode_from_slice::<#self_ty, _>(&__params, bincode::config::standard()).expect(#params_decode_expect);
                    let (method, _) = bincode::decode_from_slice::<#enum_path, _>(&__input, bincode::config::standard()).expect(#input_decode_expect);
                    #match_stmt
                }

                fn handle_http(request: http::Request) -> http::Response {
                    #[cfg(target_arch = "wasm32")]
                    set_up_panic_hook();
                    let uri = request.uri();
                    let mut path = uri.path();
                    let body = &request.body;

                    #json_match_stmt
                }
            }
        }

    };

    output.into()
}
