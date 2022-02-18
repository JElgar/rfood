use syn::{Item, ItemEnum};

pub fn create_enum(name: &String, variants: Vec<syn::Variant>) -> syn::Item {
    Item::Enum(
        ItemEnum {
            attrs: [].to_vec(),
            vis: syn::Visibility::Inherited,
            enum_token: syn::token::Enum{
                span: syn::__private::Span::call_site(),
            },
            ident: syn::Ident::new(name, syn::__private::Span::call_site()),
            generics: syn::Generics {
                lt_token: None,
                params: syn::punctuated::Punctuated::from_iter(Vec::new() as Vec<syn::GenericParam>),
                gt_token: None,
                where_clause: None,
            },
            brace_token: syn::token::Brace{
                span: syn::__private::Span::call_site(),
            },
            variants: syn::punctuated::Punctuated::from_iter(variants),
        },
    )
}

pub fn create_enum_variant(name: &String, mut fields: syn::Fields) -> syn::Variant {
    // Remove pub from fields
    if let syn::Fields::Named(enum_fields) = &fields {
    
        let new_named = &syn::punctuated::Punctuated::from_iter(
            // Remove pub from fields
            enum_fields.named.iter().map(|field| {
                match field.vis {
                    syn::Visibility::Public(_) => {
                        syn::Field{
                            vis: syn::Visibility::Inherited,
                            ..field.clone()
                        }
                    }
                    _ => field.clone(),
                }
            // Remove dyn from fields
            }).map(|mut field| {
                // Match on any box type
                match &mut field {
                    syn::Field{ty: syn::Type::Path(syn::TypePath{path: syn::Path{ref mut segments, ..}, ..}), ..} => { 
                        if let Some(segment) = segments.iter_mut().next() {
                            if segment.ident.to_string() == "Box" {
                                if let syn::PathArguments::AngleBracketed(angle_bracket_args) = &segment.arguments {
                                    let new_args: syn::punctuated::Punctuated<syn::GenericArgument, syn::token::Comma> = syn::punctuated::Punctuated::from_iter(angle_bracket_args.args.iter().map(|arg| {
                                        match arg {
                                            syn::GenericArgument::Type(
                                                syn::Type::TraitObject(
                                                    syn::TypeTraitObject{
                                                        dyn_token: Some(_),
                                                        bounds, 
                                                        ..
                                                    }
                                                )
                                            ) => {
                                                if let Some(syn::TypeParamBound::Trait(
                                                    syn::TraitBound{path, ..}
                                                )) = bounds.iter().next() {
                                                    return syn::GenericArgument::Type(
                                                        syn::Type::Path(
                                                            syn::TypePath{
                                                                qself: None,
                                                                path: path.clone(),
                                                            }
                                                        )
                                                    )
                                                }
                                                panic!("Unsupported type trait bound");
                                            },
                                            _ => panic!("Unsupported type trait bound"),
                                        }
                                    }));
                                    segment.arguments = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments{args: new_args, ..*angle_bracket_args});
                                }
                            }
                        }
                        field
                    }
                    _ => field
                }
            })
        );

        fields = syn::Fields::Named(syn::FieldsNamed{named: new_named.clone(), ..*enum_fields});
    }

    syn::Variant{
        attrs: Vec::new() as Vec<syn::Attribute>,
        ident: syn::Ident::new(name, syn::__private::Span::call_site()),
        fields,
        discriminant: None,
    }
}

pub fn create_function(sig: syn::Signature, stmts: Vec<syn::Stmt>) -> syn::Item {
    syn::Item::Fn(
        syn::ItemFn{
            sig,
            vis: syn::Visibility::Inherited,
            attrs: Vec::new() as Vec<syn::Attribute>,
            block: Box::new(syn::Block{
                brace_token: syn::token::Brace{span: syn::__private::Span::call_site()},
                stmts,
            })
        },
    )
}

pub fn create_match_statement(match_ident: syn::Ident, arms: Vec<syn::Arm>) -> syn::Expr {
    syn::Expr::Match(
        syn::ExprMatch{
            attrs: Vec::new() as Vec<syn::Attribute>,
            match_token: syn::token::Match{span: syn::__private::Span::call_site()},
            expr: Box::new(syn::Expr::Path(
                syn::ExprPath{
                    attrs: Vec::new() as Vec<syn::Attribute>,
                    qself: None,
                    path: syn::Path{
                        leading_colon: None,
                        segments: syn::punctuated::Punctuated::from_iter(
                            [
                                syn::PathSegment{
                                    ident: match_ident,
                                    arguments: syn::PathArguments::None,
                                }
                            ]
                        ),
                    },
                }
            )),
            arms,
            brace_token: syn::token::Brace{span: syn::__private::Span::call_site()},
        },
    )
}

pub fn create_match_path_for_enum(enum_ident: &String, variant_ident: &String) -> syn::Path {
    syn::Path{
        leading_colon: None,
        segments: syn::punctuated::Punctuated::from_iter(
            [
                syn::PathSegment{
                    ident: syn::Ident::new(enum_ident, syn::__private::Span::call_site()),
                    arguments: syn::PathArguments::None,
                },
                syn::PathSegment{
                    ident: syn::Ident::new(variant_ident, syn::__private::Span::call_site()),
                    arguments: syn::PathArguments::None,
                },
            ]
        ),
    }
}

pub fn create_match_arm(match_path: syn::Path, elems: Vec<syn::Ident>, body: syn::Expr) -> syn::Arm {
  syn::Arm {
    attrs: Vec::new() as Vec<syn::Attribute>,
    pat: syn::Pat::TupleStruct(syn::PatTupleStruct{
        attrs: Vec::new() as Vec<syn::Attribute>,
        path: match_path,
        pat: syn::PatTuple {
            attrs: Vec::new() as Vec<syn::Attribute>,
            paren_token: syn::token::Paren{span: syn::__private::Span::call_site()},
            elems: syn::punctuated::Punctuated::from_iter(elems.iter().map(|item| {
              syn::Pat::Ident(syn::PatIdent{
                attrs: Vec::new() as Vec<syn::Attribute>,
                by_ref: None,
                mutability: None,
                subpat: None,
                ident: item.clone(),
              })
            })),
        },
    }),
    guard: None,
    fat_arrow_token: syn::token::FatArrow{spans: [syn::__private::Span::call_site(), syn::__private::Span::call_site()]},
    body: Box::new(body),
    comma: None,
  } 
}

