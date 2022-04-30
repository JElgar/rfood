use syn::*;
use syn::punctuated::Punctuated;
use syn::__private::Span;
use syn::token::{Comma, Colon};

use crate::context::delta::{GetDeltaType, RefType, get_ident_from_path};
use crate::context::gamma::Gamma;

pub fn create_enum(name: &Ident, variants: Vec<syn::Variant>, generics: &syn::Generics, vis: Visibility) -> ItemEnum {
    ItemEnum {
        attrs: [].to_vec(),
        vis,
        enum_token: token::Enum{
            span: Span::call_site(),
        },
        ident: name.clone(),
        generics: generics.clone(),
        brace_token: syn::token::Brace{
            span: syn::__private::Span::call_site(),
        },
        variants: syn::punctuated::Punctuated::from_iter(variants),
    }
}

pub fn create_trait(name: &Ident, items: &Vec<TraitItem>, vis: Visibility) -> ItemTrait {
    ItemTrait {
        attrs: Vec::new(),
        vis,
        unsafety: None,
        auto_token: None,
        trait_token: token::Trait::default(),
        ident: name.clone(),
        generics: Generics {
            lt_token: None,
            params: Punctuated::new(),
            gt_token: None,
            where_clause: None,
        },
        colon_token: None,
        supertraits: Punctuated::new(),
        brace_token: token::Brace::default(),
        items: items.clone(),
    }
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

pub fn create_dyn_box_of_path(path: &Path) -> Path {
    Path{
        leading_colon: None,
        segments: Punctuated::from_iter(
            vec![
                PathSegment{
                    ident: Ident::new("Box", Span::call_site()),
                    arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments{
                        args: Punctuated::from_iter(
                            vec![
                                GenericArgument::Type(
                                    Type::TraitObject(
                                        TypeTraitObject{
                                            dyn_token: Some(syn::Token![dyn](Span::call_site())),
                                            bounds: Punctuated::from_iter(
                                                vec![
                                                    TypeParamBound::Trait(
                                                        TraitBound{
                                                            lifetimes: None,
                                                            path: path.clone(),
                                                            modifier: TraitBoundModifier::None,
                                                            paren_token: None,
                                                        }
                                                    )
                                                ]
                                            ),
                                        }
                                    )
                                )
                            ]
                        ),
                        lt_token: token::Lt::default(),
                        gt_token: token::Gt::default(),
                        colon2_token: Some(token::Colon2::default()),
                    })
                }
            ]
        ),
    }
}

pub fn create_dyn_box_of_type(type_: &Type) -> Type {
    match type_ {
        Type::Path(type_path) => {
            Type::Path(TypePath{
                path: create_dyn_box_of_path(&type_path.path),
                ..type_path.clone()
            })
        },
        _ => panic!("Unsupported type"),
    }
}

pub fn create_dyn_box_arg(fn_arg: &FnArg) -> FnArg {
    match fn_arg {
        FnArg::Typed(typed) => {
            FnArg::Typed(PatType{
                ty: Box::new(create_dyn_box_of_type(&*typed.ty)),
                ..typed.clone()
            })
        },
        _ => panic!("Unsupported fn arg type for creating dyn box")
    }
}

pub fn create_struct(ident: &Ident, trait_ident: &Ident, mut fields: Fields, vis: Visibility) -> ItemStruct {
    // TODO remove the mutability here?
    let new_fields = match &mut fields {
        Fields::Named(FieldsNamed { named: fields, .. }) | Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. }) => {
            Fields::Named(FieldsNamed{
               brace_token: token::Brace::default(),
               named: Punctuated::from_iter(fields.iter_mut().map(|field| {
                   // Set the field to public 
                   field.vis = Visibility::Public(VisPublic{pub_token: token::Pub::default()});
                   field.clone()
               }))
           })
        },
        Fields::Unit => fields
    };

    ItemStruct {
        attrs: Vec::new(),
        vis,
        struct_token: token::Struct::default(),
        fields: new_fields,
        ident: ident.clone(),
        generics: Generics::default(),
        semi_token: Some(token::Semi::default()),
    }
}

pub fn create_impl(trait_ident: &Ident, struct_ident: &Ident, items: Vec<ImplItem>) -> ItemImpl {
    ItemImpl {
        attrs: Vec::new(),
        brace_token: token::Brace::default(),
        defaultness: None,
        generics: Generics::default(),
        impl_token: token::Impl::default(),
        items,
        trait_: Some((
            None,
            trait_ident.clone().into(),
            token::For::default()
        )),
        self_ty: Box::new(
            Type::Path(TypePath{
                qself: None,
                path: struct_ident.clone().into(),
            })
        ),
        unsafety: None,
    }
}

pub fn create_impl_method(sig: &Signature, block: &Block) -> ImplItemMethod {
    ImplItemMethod {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        defaultness: None,
        sig: sig.clone(),
        block: block.clone(),
    }
}

pub fn create_function(sig: Signature, stmts: Vec<Stmt>, vis: Visibility) -> ItemFn {
   syn::ItemFn{
       sig,
       vis,
       attrs: Vec::new() as Vec<syn::Attribute>,
       block: Box::new(syn::Block{
           brace_token: syn::token::Brace{span: syn::__private::Span::call_site()},
           stmts,
       }),
   }
}

pub fn create_match_statement(match_ident: &syn::Ident, arms: Vec<syn::Arm>) -> syn::Expr {
    syn::Expr::Match(
        syn::ExprMatch{
            attrs: Vec::new() as Vec<syn::Attribute>,
            match_token: syn::token::Match{span: syn::__private::Span::call_site()},
            expr: Box::new(create_reference_of_expr(&Expr::Path(
                syn::ExprPath{
                    attrs: Vec::new() as Vec<syn::Attribute>,
                    qself: None,
                    path: match_ident.clone().into(),
                }
            ))),
            arms,
            brace_token: syn::token::Brace{span: syn::__private::Span::call_site()},
        },
    )
}

pub fn create_expr_path_from_path(path: Path) -> ExprPath {
    ExprPath { 
        attrs: Vec::new(),
        qself: None,
        path: path.clone(),
    }
}

pub fn create_path_from_ident(ident: &Ident) -> Path {
    syn::Path{
        leading_colon: None,
        segments: syn::punctuated::Punctuated::from_iter(
            [
                syn::PathSegment{
                    ident: ident.clone(),
                    arguments: syn::PathArguments::None,
                },
            ]
        ),
    }
}

pub fn create_expr_from_ident(ident: &Ident) -> Expr {
    Expr::Path(
        create_expr_path_from_path(create_path_from_ident(ident))
    )
}

pub fn create_path_for_enum(enum_ident: &Ident, variant_ident: &Ident) -> syn::Path {
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

pub fn create_wildcard_match_arm(body: syn::Expr) -> syn::Arm {
    syn::Arm{
        attrs: Vec::new() as Vec<syn::Attribute>,
        pat: Pat::Wild(
            syn::PatWild{
                attrs: Vec::new() as Vec<syn::Attribute>,
                underscore_token: token::Underscore::default(),
            }
        ),
        guard: None,
        fat_arrow_token: syn::token::FatArrow::default(),
        body: Box::new(body),
        comma: None,
    }
}

pub fn create_match_arm(match_path: syn::Path, elems: Vec<syn::Ident>, body: syn::Expr, mutable: bool) -> syn::Arm {

  // Replace any call to self in the body and remember which methods are used

  syn::Arm {
    attrs: Vec::new() as Vec<syn::Attribute>,
    pat: syn::Pat::Struct(syn::PatStruct{
        attrs: Vec::new() as Vec<syn::Attribute>,
        path: match_path,
        fields: Punctuated::from_iter(
            elems.iter().map(|item| {
                syn::FieldPat{
                    attrs: Vec::new() as Vec<syn::Attribute>,
                    colon_token: None,
                    member: Member::Named(item.clone()),
                    pat: Box::new(Pat::Ident(
                        syn::PatIdent{
                            attrs: Vec::new() as Vec<syn::Attribute>,
                            by_ref: None,
                            mutability: if mutable {Some(token::Mut::default())} else {None},
                            ident: item.clone(),
                            subpat: None,
                        }
                    ))  
                }
            })
        ),
        brace_token: syn::token::Brace{span: syn::__private::Span::call_site()},
        dot2_token: None,
        // pat: syn::PatTuple {
        //     attrs: Vec::new() as Vec<syn::Attribute>,
        //     paren_token: syn::token::Paren{span: syn::__private::Span::call_site()},
        //     elems: syn::punctuated::Punctuated::from_iter(elems.iter().map(|item| {
        //       syn::Pat::Ident(syn::PatIdent{
        //         attrs: Vec::new() as Vec<syn::Attribute>,
        //         by_ref: None,
        //         mutability: None,
        //         subpat: None,
        //         ident: item.clone(),
        //       })
        //     })),
        // },
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

pub fn create_dereference_of_expr(expr: &Expr) -> Expr {
    Expr::Unary(ExprUnary{
        attrs: Vec::new(),
        expr: Box::new(expr.clone()),
        op: UnOp::Deref(token::Star { spans: [Span::call_site()] }),
    })
}

/// If the provided expression is a deference, *something remove the dereference and return the
/// inner expression. Otherwise, return the original expression.
pub fn remove_deference_of_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Unary(ExprUnary{
            op: UnOp::Deref{..},
            box expr,
            ..
        }) => expr.clone(),
        _ => expr.clone(),
    }
}

pub fn remove_reference_of_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Reference(ExprReference{box expr, ..}) => expr.clone(),
        _ => expr.clone(),
    }
}

pub fn create_reference_of_expr(expr: &Expr) -> Expr {
    Expr::Reference(
        ExprReference{
            attrs: Vec::new(),
            and_token: token::And { spans: [Span::call_site()] },
            mutability: None,
            raw: syn::reserved::Reserved::default(),
            expr: Box::new(expr.clone()),
        }
    )
}

pub fn create_box_of_expr(expr: &Expr) -> Expr {
    Expr::Call(
        ExprCall {
            attrs: Vec::new(),
            paren_token: syn::token::Paren{span: syn::__private::Span::call_site()},
            args: Punctuated::from_iter(
                vec![expr.clone()]
            ),
            func: Box::new(Expr::Path(
                syn::ExprPath{
                    attrs: Vec::new(),
                    qself: None,
                    path: syn::Path{
                        leading_colon: None,
                        segments: syn::punctuated::Punctuated::from_iter(
                            [
                                syn::PathSegment{
                                    arguments: syn::PathArguments::None,
                                    ident: Ident::new("Box", Span::call_site()),
                                },
                                syn::PathSegment{
                                    arguments: syn::PathArguments::None,
                                    ident: Ident::new("new", Span::call_site()),
                                }
                            ]
                        )
                    }
                }
            )),
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

pub fn generic_parameter_from_generic_argument(generic_argument: &GenericArgument) -> GenericParam {
    if let GenericArgument::Type(Type::Path(TypePath {path, ..})) = generic_argument {
        return GenericParam::Type(TypeParam{
            attrs: Vec::new(),
            ident: path.segments.last().unwrap().ident.clone(),
            colon_token: None,
            bounds: Punctuated::new(),
            eq_token: None,
            default: None,
        });
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

pub fn create_consumer_signature_arg(enum_name: &Ident, enum_instance_name: &Ident, reference: bool, enum_generics: &Generics) -> syn::FnArg {

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

pub fn create_self_fn_arg(reference_type: RefType) -> FnArg {
    if matches!(reference_type, RefType::Box(_)) {
        FnArg::Typed(
            syn::PatType{
                attrs: Vec::new(),
                colon_token: Colon::default(),
                pat: Box::new(
                    Pat::Ident(PatIdent{
                        attrs: [].to_vec(),
                        by_ref: None,
                        mutability: None,
                        ident: Ident::new("self", Span::call_site()),
                        subpat: None,
                    })
                ),
                ty: Box::new(
                    Type::Path(
                        TypePath{
                            qself: None,
                            path: syn::Path{
                                leading_colon: None,
                                segments: syn::punctuated::Punctuated::from_iter(
                                    [
                                        syn::PathSegment{
                                            ident: Ident::new("Box", Span::call_site()),
                                            arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments{
                                                colon2_token: None,
                                                lt_token: token::Lt::default(),
                                                gt_token: token::Gt::default(),
                                                args: Punctuated::from_iter(
                                                    vec![
                                                        GenericArgument::Type(Type::Path(
                                                            TypePath{
                                                                qself: None,
                                                                path: syn::Path{
                                                                    leading_colon: None,
                                                                    segments: syn::punctuated::Punctuated::from_iter(
                                                                        [
                                                                            syn::PathSegment{
                                                                                ident: Ident::new("Self", Span::call_site()),
                                                                                arguments: PathArguments::None,
                                                                            }
                                                                        ]
                                                                    )
                                                                }
                                                            }
                                                        ))
                                                    ]
                                                ) 
                                            }),
                                        },
                                    ]
                                )
                            }
                        }
                    )
                )
            }
        )
    } else {
        FnArg::Receiver(
            Receiver{
                attrs: Vec::new(),
                reference: match reference_type {
                    RefType::Ref(_) => Some((token::And::default(), None)),
                    _ => None, 
                },
                mutability: None,
                self_token: token::SelfValue::default(),
            }
        )
    }
}

pub fn create_self_path() -> Path {
    Path{
        leading_colon: None,
        segments: Punctuated::from_iter(
            [
                PathSegment{
                    ident: Ident::new("self", Span::call_site()),
                    arguments: PathArguments::None,
                }
            ]
        )
    }
}

pub fn create_self_expr() -> Expr {
    Expr::Path(
        ExprPath{
            attrs: Vec::new(),
            qself: None,
            path: create_self_path(),
        }
    )
}

pub fn create_field_call(base_name: &Ident, field_name: &Ident) -> Expr {
    Expr::Field(ExprField{
        attrs: Vec::new() as Vec<syn::Attribute>,
        base: Box::new(Expr::Path(
            ExprPath {
                attrs: Vec::new() as Vec<syn::Attribute>,
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter([syn::PathSegment {
                        ident: base_name.clone(),
                        arguments: syn::PathArguments::None
                    }])
                }
            }
        )),
        member: syn::Member::Named(field_name.clone()),
        dot_token: token::Dot::default(),
    })
}

pub fn create_self_field_call(field_name: &Ident) -> Expr {
    create_field_call(&Ident::new("self", Span::call_site()), field_name)
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

pub fn create_self_method_call(method: &Ident, args: Punctuated<Expr, Comma>) -> Expr {
    create_method_call(method, &create_self_expr(), &args)
}

pub fn add_self_to_path(exp: &Expr) -> Expr {
    match exp {
        Expr::Path(expr_path) | Expr::Reference(ExprReference { expr: box Expr::Path(expr_path), .. }) => {
            create_self_field_call(&get_ident_from_path(&expr_path.path))
        },
        _ => panic!("Unsupported expression type for adding self, {:?}", exp),
    }
}

pub fn create_method_call(method: &Ident, receiver: &Expr, args: &Punctuated<Expr, Comma>) -> Expr {
    Expr::MethodCall(ExprMethodCall{
        attrs: Vec::new(),
        receiver: Box::new(receiver.clone()),
        dot_token: token::Dot { spans: [Span::call_site()] },
        method: method.clone(),
        args: args.clone(),
        paren_token: token::Paren::default(),
        turbofish: None,
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

pub fn create_let_stmt(name: &Ident, expr: &Expr, mutable: bool) -> Local {
    Local{
        attrs: Vec::new(),
        let_token: token::Let::default(),
        init: Some((token::Eq::default(), Box::new(expr.clone()))),
        semi_token: token::Semi::default(),
        pat: Pat::Ident(
            PatIdent{
                attrs: Vec::new(),
                ident: name.clone(),
                mutability: if mutable {Some(token::Mut::default())} else {None},
                subpat: None,
                by_ref : None,
            }
        )
    }
}

pub fn create_assignment_expr(receiver: Expr, expr: Expr) -> ExprAssign {
    ExprAssign{
        attrs: Vec::new(),
        eq_token: token::Eq::default(),
        left: Box::new(receiver),
        right: Box::new(expr),
    }
}

/// Add expression into a block at given index. 
pub fn add_stmts_to_block(stmt: &Stmt, block: &Block, index: usize) -> Block {
    let mut stmts = block.stmts.clone();
    stmts.insert(index, stmt.clone());
    Block{
        stmts,
        ..block.clone()
    }
}
