use proc_macro2::{Ident, Span};
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

pub fn machine(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut input = parse2::<syn::ItemImpl>(input).expect("Can only apply #[carol] to impl");
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

    for item in &mut input.items {
        if let ImplItem::Method(method) = item {
            if !method
                .attrs
                .iter()
                .any(|attr| attr.path.to_token_stream().to_string() == "activate")
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
            let mut inputs = method.sig.inputs.iter_mut().peekable();
            let mut _has_receiver = false; //TODO use this to only pass in receiver if it's there
            if let Some(syn::FnArg::Receiver(_)) = inputs.peek() {
                _has_receiver = true;
                let _ = inputs.next();
            } else {
                return quote_spanned!(sig_span => compile_error!("the first argument to an #[activate] method must be &self"));
            }

            if let Some(syn::FnArg::Typed(fn_arg)) = inputs.peek() {
                if let arg @ syn::Pat::Ident(pat_ident) = &*fn_arg.pat {
                    let ident = pat_ident.ident.to_string();
                    if ident == "cap" || ident == "_cap" {
                        let _ = inputs.next();
                    } else {
                        return quote_spanned!(arg.span() => compile_error!("the second argument after `&self` to an #[activate] method must be `cap`"));
                    }
                }
            }

            for fn_arg in inputs {
                let span = fn_arg.span();
                let (field, match_field) = match fn_arg {
                    syn::FnArg::Typed(fn_arg) => match fn_arg.pat.as_mut() {
                        syn::Pat::Ident(pat_ident) => {
                            let mut attrs = vec![];
                            for attr in fn_arg.attrs.drain(..) {
                                if attr.path.get_ident().map(|ident| ident.to_string())
                                    == Some("with_serde".into())
                                {
                                    attrs.push(parse_quote!(#[bincode(with_serde)]));
                                } else {
                                    return quote_spanned!(attr.span() => compile_error!("only 'with_serde' is a valid function argument attribute"));
                                }
                            }
                            (
                                Field {
                                    attrs,
                                    vis: syn::Visibility::Public(VisPublic {
                                        pub_token: Token![pub](span),
                                    }),
                                    ident: Some(pat_ident.ident.clone()),
                                    colon_token: Some(fn_arg.colon_token),
                                    ty: *fn_arg.ty.clone(),
                                },
                                FieldPat {
                                    member: syn::Member::Named(pat_ident.ident.clone()),
                                    colon_token: None,
                                    pat: fn_arg.pat.clone(),
                                    attrs: vec![],
                                },
                            )
                        }
                        arg => {
                            return quote_spanned!(arg.span() => compile_error!("#[activate] only takes plain fn arguments"))
                        }
                    },
                    _ => unreachable!("we dealt with receiver already"),
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

            let activate_call = ExprMethodCall {
                attrs: vec![],
                receiver: Box::new(Expr::Verbatim(quote! { machine })),
                dot_token: Token![.](sig_span),
                method: method.sig.ident.clone(),
                turbofish: None,
                paren_token: token::Paren::default(),
                args: {
                    let mut punctuated = Punctuated::new();
                    punctuated.push(parse_quote! { &__ctx });
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

            let route = format!("/activate/{}", method.sig.ident);

            let handle_output = match method.sig.output.clone() {
                ReturnType::Default => quote_spanned! { method.sig.span() => http::Response {
                    headers: vec![],
                    status: 204,
                    body: vec![]
                }},
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
                struct_path.to_token_stream().to_string().replace(' ', "")
            );
            let bincode_encode_error =
                format!("#[carol] bincode encoding input to {}", method_name);
            json_match_arms.push(Arm {
                attrs: vec![],
                pat,
                fat_arrow_token: Token![=>](sig_span),
                body: Box::new(Expr::Verbatim(quote_spanned! { sig_span => {
                        use carol_guest::{bincode, serde_json};
                        let method_struct = carol_guest::serde_json::from_slice::<#struct_path>(body).expect(#json_decode_error);
                        let method_variant = #variant_path(method_struct);
                        let binary_input: Vec<u8> = carol_guest::bincode::encode_to_vec(&method_variant, carol_guest::bincode::config::standard()).expect(#bincode_encode_error);
                        let output = match carol_guest::machines::Cap::self_activate(&__ctx, &binary_input) {
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
        self_ty.to_token_stream().to_string().replace(' ', "")
    );
    let enum_path: Path = parse_quote!(#carol_mod::#enum_name);
    let input_decode_expect = format!(
        "#[carol] bincode decoding input as {}",
        enum_path.to_token_stream().to_string().replace(' ', "")
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
                    carol_guest::bind::carol::machine::log::set_panic_message(&panic_info.to_string());
                    (original_hook)(panic_info)
                }));
            }

            use carol_guest::{http, bincode};
            impl carol_guest::bind::exports::machine::Machine for #self_ty {
                fn activate(__params: Vec<u8>, __input: Vec<u8>) -> Vec<u8> {
                    #[cfg(target_arch = "wasm32")]
                    set_up_panic_hook();
                    let __ctx = carol_guest::ActivateCap;
                    let (machine, _) = bincode::decode_from_slice::<#self_ty, _>(&__params, bincode::config::standard()).expect(#params_decode_expect);
                    let (method, _) = bincode::decode_from_slice::<#enum_path, _>(&__input, bincode::config::standard()).expect(#input_decode_expect);
                    #match_stmt
                }

                fn handle_http(request: http::Request) -> http::Response {
                    #[cfg(target_arch = "wasm32")]
                    set_up_panic_hook();
                    let __ctx = carol_guest::HttpHandlerCap;

                    let uri = request.uri();
                    let mut path = uri.path();
                    let body = &request.body;

                    #json_match_stmt
                }
            }
        }

    };

    output
}
