use syn::*;
use syn::punctuated::Punctuated;
use syn::__private::Span;
use syn::token::{Comma, Colon};

pub fn create_enum(name: &Ident, variants: Vec<syn::Variant>, generics: &syn::Generics) -> syn::Item {
    Item::Enum(
        ItemEnum {
            attrs: [].to_vec(),
            vis: Visibility::Inherited,
            enum_token: token::Enum{
                span: Span::call_site(),
            },
            ident: name.clone(),
            generics: generics.clone(),
            brace_token: syn::token::Brace{
                span: syn::__private::Span::call_site(),
            },
            variants: syn::punctuated::Punctuated::from_iter(variants),
        },
    )
}

pub fn create_enum_variant(name: &Ident, mut fields: syn::Fields) -> syn::Variant {
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
        ident: name.clone(),
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
            }),
        },
    )
}

pub fn create_match_statement(match_ident: &syn::Ident, arms: Vec<syn::Arm>) -> syn::Expr {
    syn::Expr::Match(
        syn::ExprMatch{
            attrs: Vec::new() as Vec<syn::Attribute>,
            match_token: syn::token::Match{span: syn::__private::Span::call_site()},
            expr: Box::new(syn::Expr::Path(
                syn::ExprPath{
                    attrs: Vec::new() as Vec<syn::Attribute>,
                    qself: None,
                    path: match_ident.clone().into(),
                }
            )),
            arms,
            brace_token: syn::token::Brace{span: syn::__private::Span::call_site()},
        },
    )
}

pub fn create_match_path_for_enum(enum_ident: &Ident, variant_ident: &Ident) -> syn::Path {
    syn::Path{
        leading_colon: None,
        segments: syn::punctuated::Punctuated::from_iter(
            [
                syn::PathSegment{
                    ident: enum_ident.clone(),
                    arguments: syn::PathArguments::None,
                },
                syn::PathSegment{
                    ident: variant_ident.clone(),
                    arguments: syn::PathArguments::None,
                },
            ]
        ),
    }
}

pub fn create_match_arm(match_path: syn::Path, elems: Vec<syn::Ident>, body: syn::Expr) -> syn::Arm {

  // Replace any call to self in the body and remember which methods are used

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

pub fn create_reference_of_type(type_: Type) -> Type {
    Type::Reference(
        TypeReference{
            and_token: token::And { spans: [Span::call_site()] },
            lifetime: None,
            mutability: None,
            elem: Box::new(type_.clone()),
        }
    )
}

pub fn generic_argumnet_from_generic_parameter(generic_param: GenericParam) -> GenericArgument {
    if let GenericParam::Type(type_param) = generic_param {
        return GenericArgument::Type(Type::Path(TypePath{
            qself: None,
            path: type_param.ident.clone().into(),
        }));
    }
    panic!("Unsupported generic parameter, currently only type parameters are supported");
}

pub fn add_generics_to_path_segment(segmenet: PathSegment, generics: &syn::Generics) -> PathSegment {
    let arguments = PathArguments::AngleBracketed(AngleBracketedGenericArguments{
        colon2_token: None,
        lt_token: token::Lt { spans: [Span::call_site()] },
        gt_token: token::Gt { spans: [Span::call_site()] },
        args: generics.params.iter().map(|generic_parm| generic_argumnet_from_generic_parameter(generic_parm.clone())).collect(),
    });
    PathSegment { arguments, ..segmenet }
}

pub fn create_consumer_signature(enum_name: &Ident, enum_instance_name: &Ident, reference: bool, enum_generics: &Generics) -> syn::FnArg {

    let path_segment = add_generics_to_path_segment(PathSegment{
        ident: enum_name.clone(),
        arguments: PathArguments::None,
    }, enum_generics);

    let mut type_ = Type::Path(
        TypePath{
            qself: None,
            path: path_segment.into(),
        }
    );

    if reference {
        type_ = create_reference_of_type(type_);
    }

    return syn::FnArg::Typed(
        syn::PatType{
            attrs: Vec::new(),
            colon_token: Colon{
                spans: [Span::call_site()],
            },
            pat: Box::new(
                Pat::Ident(PatIdent{
                    attrs: [].to_vec(),
                    by_ref: None,
                    mutability: None,
                    ident: enum_instance_name.clone(),
                    subpat: None,
                })
            ),
            ty: Box::new(type_)
        }
    )
}

pub fn create_function_call(method: &Ident, args: Punctuated<Expr, Comma>) -> Expr {
    let method_path = Expr::Path(ExprPath{attrs: Vec::new(), qself: None, path: Path{
        leading_colon: None,
        segments: Punctuated::from_iter(
            vec![PathSegment{ident: method.clone(), arguments: PathArguments::None}]
        ),
    }});

    Expr::Call(ExprCall{
        attrs: Vec::new(),
        paren_token: token::Paren { span: Span::call_site() },
        func: Box::new(method_path),
        args: args.clone(),
    })
}

pub fn create_expr_path_to_ident(ident: &Ident) -> ExprPath {
    return ExprPath{
        attrs: Vec::new(),
        qself: None,
        path: Path {
            leading_colon: None,
            segments: Punctuated::from_iter(vec![
                PathSegment{
                    ident: ident.clone(),
                    arguments: PathArguments::None,
                }
            ])
        }
    }
}

/// Given an ident create a type for it
pub fn create_type_from_ident(ident: &Ident) -> Type {
    Type::Path(TypePath{
        qself: None,
        path: ident.clone().into(),
    })
}

pub fn create_return_type_from_ident(ident: &Ident) -> ReturnType {
    ReturnType::Type(
        token::RArrow { spans: [Span::call_site(), Span::call_site()] },
        Box::new(create_type_from_ident(ident))
    )
}

pub fn create_expression_block(stmts: Vec<syn::Stmt>) -> Expr {
    Expr::Block(ExprBlock{
        attrs: Vec::new(),
        label: None,
        block: Block {
            brace_token: syn::token::Brace{
                span: syn::__private::Span::call_site(),
            },
            stmts
        }
    })
}
