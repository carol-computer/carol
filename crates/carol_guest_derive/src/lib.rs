use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse2,
    punctuated::Punctuated,
    token::{self, Brace},
    Arm, AttrStyle, Attribute, Expr, ExprMatch, ExprMethodCall, ExprPath, Field, FieldPat,
    FieldsNamed, Generics, ImplItem, ItemEnum, Pat, PatStruct, Path, PathSegment, Token, Variant,
    VisPublic,
};

#[proc_macro_attribute]
pub fn carol_contract(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let output = contract_inner(input);
    proc_macro::TokenStream::from(output)
}

fn contract_inner(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let input = parse2::<syn::ItemImpl>(input).unwrap();
    let enum_name = match *input.self_ty.clone() {
        syn::Type::Path(path) => {
            let ident = path.path.get_ident().unwrap().clone();
            Ident::new(&format!("{}Methods", ident), Span::call_site())
        }
        _ => panic!("can only derive #[carol_contract] on a path"),
    };

    let mut call_interface_enum = ItemEnum {
        attrs: vec![Attribute {
            pound_token: Token![#](Span::call_site()),
            style: AttrStyle::Outer,
            bracket_token: token::Bracket::default(),
            path: Path::from(PathSegment::from(Ident::new(
                &format!("derive"),
                Span::call_site(),
            ))),
            tokens: quote! { (carol_guest::bincode::Decode, carol_guest::bincode::Encode) },
        }],
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

    for item in input.items.clone() {
        if let ImplItem::Method(method) = item {
            let mut fields = FieldsNamed {
                brace_token: Brace::default(),
                named: Punctuated::default(),
            };
            let mut match_fields = Punctuated::default();

            for fn_arg in method.sig.inputs {
                let (field, match_field) = match fn_arg {
                    syn::FnArg::Receiver(_) => continue,
                    syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => (
                            Field {
                                attrs: vec![],
                                vis: syn::Visibility::Inherited,
                                ident: Some(pat_ident.ident.clone()),
                                colon_token: Some(pat_type.colon_token),
                                ty: *pat_type.ty,
                            },
                            FieldPat {
                                attrs: vec![],
                                member: syn::Member::Named(pat_ident.ident),
                                colon_token: None,
                                pat: pat_type.pat,
                            },
                        ),
                        _ => panic!("only take ident fn args"),
                    },
                };

                fields.named.push(field);
                match_fields.push(match_field);
            }

            let variant = Variant {
                attrs: Default::default(),
                ident: Ident::new(
                    &heck::AsUpperCamelCase(&method.sig.ident.to_string()).to_string(),
                    method.sig.ident.span(),
                ),
                fields: syn::Fields::Named(fields.clone()),
                discriminant: None,
            };

            let variant_path = {
                let mut puncuated = Punctuated::new();
                puncuated.push(PathSegment::from(enum_name.clone()));
                puncuated.push(PathSegment::from(variant.ident.clone()));
                Path {
                    leading_colon: None,
                    segments: puncuated,
                }
            };

            let contract_call = ExprMethodCall {
                attrs: vec![],
                receiver: Box::new(Expr::Verbatim(quote! { contract })),
                dot_token: Token![.](Span::call_site()),
                method: method.sig.ident,
                turbofish: None,
                paren_token: token::Paren::default(),
                args: {
                    let mut punctuated = Punctuated::new();
                    for field in fields.named {
                        punctuated.push(Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path::from(PathSegment::from(field.ident.unwrap())),
                        }));
                    }
                    punctuated
                },
            };

            let decode_call = quote! { carol_guest::bincode::encode_to_vec(#contract_call, carol_guest::bincode::config::standard()).unwrap() };

            match_arms.push(Arm {
                attrs: vec![],
                pat: Pat::Struct(PatStruct {
                    attrs: vec![],
                    path: variant_path,
                    brace_token: token::Brace::default(),
                    fields: match_fields,
                    dot2_token: None,
                }),
                guard: None,
                fat_arrow_token: Token![=>](Span::call_site()),
                body: Box::new(Expr::Verbatim(decode_call)),
                comma: None,
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

    let self_ty = input.self_ty.clone();
    let output = quote! {

        impl carol_guest::contract::Contract for #self_ty {
            fn activate(params: Vec<u8>, exec_input: Vec<u8>) -> Vec<u8> {
                let (contract, _) = carol_guest::bincode::decode_from_slice::<#self_ty, _>(&params, carol_guest::bincode::config::standard()).unwrap();
                let (method, _) = carol_guest::bincode::decode_from_slice::<#enum_name, _>(&exec_input, carol_guest::bincode::config::standard()).unwrap();
                #match_stmt
            }
        }

        #call_interface_enum


        #input
    };

    output.into()
}
