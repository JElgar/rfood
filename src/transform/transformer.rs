use ast::print::write_and_fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit::*;
use syn::visit_mut::*;
use syn::*;

use crate::context;
use crate::utils::utils::PopFirst;
use context::delta::*;
use context::errors::*;
use context::gamma::*;

use crate::ast;
use ast::create::*;

use crate::transform;
use transform::visitors::*;

use quote::quote;

#[derive(clap::ArgEnum, Clone)]
pub enum TransformType {
    OOPToFP,
    FPToOOP,
}

fn remove_item_from_syntax(syntax: &mut syn::File, item: syn::Item) {
    let index = syntax.items.iter().position(|sitem| *sitem == item);
    if index.is_some() {
        syntax.items.remove(index.unwrap());
    }
}

pub fn transform_file(path: &PathBuf, output_path: &PathBuf, transform_type: &TransformType) {
    //-- Do the transfrom --//
    let mut file = File::open(path).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    let mut syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    let mut transformed_syntax = syn::File {
        items: Vec::new(),
        ..syntax.clone()
    };

    // Generate global gamma context
    let mut gamma: Gamma = generate_gamma(&syntax);
    let gamma_mut_borrow = &mut gamma;

    match transform_type {
        // Stage 1
        TransformType::OOPToFP => {
            // Transform all the interfaces
            println!("Transorming all traits");

            for item in syntax.items.clone() {
                match &item {
                    Item::Trait(trait_) => {
                        // Add the transformed items to the transformed syntax
                        transformed_syntax
                            .items
                            .append(&mut transform_trait(&trait_, gamma_mut_borrow));
                    
                        // Remove the original trait from the syntax
                        for (item_struct, item_impl) in gamma_mut_borrow.get_generators(&trait_.ident) {
                            remove_item_from_syntax(&mut syntax, syn::Item::Struct(item_struct));
                            remove_item_from_syntax(&mut syntax, syn::Item::Impl(item_impl));
                        }
                        remove_item_from_syntax(&mut syntax, syn::Item::Trait(trait_.clone()));
                    },
                    // TODO 
                    // Item::Struct(struct_) => {}
                    // TODO 
                    // Item::Fn(struct_) => {}
                    _ => () 
                }
            }
            println!("Transormed all traits");
        }
        TransformType::FPToOOP => {
            // Transform all the enums
            println!("Transorming all the enums");
            
            for enum_ in gamma_mut_borrow.enums.clone() {
                // Get the consumers for the enum
                let consumers = gamma_mut_borrow.get_enum_consumers(&enum_);

                // 1st parse, transform types
                transformed_syntax
                    .items
                    .extend(transform_enum(&enum_, gamma_mut_borrow));

                // For all the consumers, for each arm create a method in each impl
                for consumer in consumers {
                    remove_item_from_syntax(&mut syntax, syn::Item::Fn(consumer.clone()));
                }
                remove_item_from_syntax(&mut syntax, syn::Item::Enum(enum_.clone()));
            }

            // Update other types
            let type_transformer = |type_| transform_type_fp(type_, &gamma);

            for item in syntax.items.iter_mut() {
                match item {
                    // If a struct is defined and the transform is FP to OOP, then fix the attrs 
                    Item::Struct(struct_) => {
                        *item = Item::Struct(ItemStruct{
                            fields: transform_type_struct_fields(
                                &struct_.fields,
                                type_transformer
                            ),
                            ..struct_.clone()
                        })
                    },
                    // Top level functions should also have types transformed
                    Item::Fn(fn_) if !gamma.is_consumer(&fn_.sig.ident) => {
                        fn_.sig = transform_singature_types(&fn_.sig, type_transformer)
                    },
                    Item::Trait(item_trait) => {
                        for item in item_trait.items.iter_mut() {
                            if let TraitItem::Method(trait_item_method) = item {
                                trait_item_method.sig = transform_singature_types(
                                    &trait_item_method.sig, type_transformer
                                ) 
                            }
                        }
                    },
                    Item::Impl(item_impl) => {
                        for item in item_impl.items.iter_mut() {
                            if let ImplItem::Method(impl_item_method) = item {
                                impl_item_method.sig = transform_singature_types(
                                    &impl_item_method.sig, type_transformer
                                ) 
                            }
                        }
                    }
                    _ => ()
                }
            }

            println!("Transormed all the enums");
        }
    }

    // Collect gamma for the transformed and untouched code
    let mut gamma = Gamma::empty();
    gamma.visit_file(&syntax);
    gamma.visit_file(&transformed_syntax);
  
    let mut delta = Delta::new();
    // Stage 2 - Transform all the new items and any untransformed items
    for item in transformed_syntax.items.iter_mut() {
        *item = transform_item(&item, &transform_type, &gamma, &mut delta)
    }
    for item in &syntax.items {
        transformed_syntax
            .items
            .push(transform_item(item, &transform_type, &gamma, &mut delta));
    }

    // Write output to file
    if write_and_fmt(output_path, quote!(#transformed_syntax)).is_err() {
        panic!("Unable to write output file");
    }
}

/// Transform a type
pub fn transform_type_fp_consumer(type_: Type, consumer: Ident) -> Type {
    todo!()
    // Borrow -> Borrow
    
    // If DT 
}

pub fn transform_type_fp(type_: Type, gamma: &Gamma) -> Type {
    let dt = type_.get_delta_type();

    if gamma.is_enum(&dt.name) {
        match dt.ref_type {
            RefType::None => create_dyn_box_of_type(&type_),
            RefType::Box(_) => create_dyn_box_of_type(
                &create_type_from_ident(&dt.name)
            ),
            _ => type_
        }
        // Create dynamic box of type
    } else {
        type_
    }
}

pub fn transform_type_struct_fields<F>(fields: &Fields, type_transformer: F) -> Fields where F: Fn(Type) -> Type {
    let mut fields = fields.clone();
    match &mut fields {
        Fields::Named(FieldsNamed{named: fields, ..}) | Fields::Unnamed(FieldsUnnamed{unnamed: fields, ..}) => {
            for field in fields.iter_mut() {
                field.ty =  type_transformer(field.ty.clone())
            }
        },
        _ => ()
    }
    fields
}

pub fn transform_singature_types<F>(sig: &Signature, type_transformer: F) -> Signature where F: Fn(Type) -> Type {
    Signature {
        inputs: Punctuated::from_iter(sig.inputs.iter().map(|input| {
            match input {
                FnArg::Typed(pat_type) => {
                    FnArg::Typed(PatType{
                        ty: Box::new(type_transformer(*pat_type.ty.clone())),
                        ..pat_type.clone()
                    })
                },
                _ => input.clone(),
            }
        })),
        output: match &sig.output {
            ReturnType::Default => ReturnType::Default,
            ReturnType::Type(r, box type_) => ReturnType::Type(r.clone(), Box::new(type_transformer(type_.clone())))
        },
        ..sig.clone()
    }
}

/// Transform a interface (trait) into a datatype (enum)
///
/// This transforms the trait it self as well as the implementations of the trait
pub fn transform_trait(trait_: &ItemTrait, gamma: &mut Gamma) -> Vec<Item> {
    // Create enum varaint for each generator of the trait
    let variants: Vec<syn::Variant> = Vec::from_iter(
        gamma
            .get_generators(&trait_.ident)
            .iter()
            .map(|(generator, _)| create_enum_variant(&generator.ident, generator.fields.clone())),
    );

    // Create the enum
    let new_enum = ast::create::create_enum(
        &trait_.ident,
        variants,
        &trait_.generics,
        trait_.vis.clone(),
    );
    gamma.add_enum(&new_enum);

    // For each destructor of the trait create a new consumer of the enum
    let mut consumers = Vec::from_iter(
        gamma
            .get_destructors(&trait_.ident)
            .iter()
            .map(|destructor| transform_destructor(trait_, destructor, &new_enum, gamma)),
    );

    let mut items = vec![Item::Enum(new_enum.clone())];
    items.append(&mut consumers);
    return items;
}

pub fn transform_consumer_fn_to_trait_item(consumer: &ItemFn, gamma: &mut Gamma) -> TraitItemMethod {
    TraitItemMethod {
        attrs: consumer.attrs.clone(),
        sig: transform_consumer_signature(&consumer.sig, gamma),
        default: None,
        semi_token: Some(token::Semi::default()),
    }
}

pub fn transform_enum(enum_: &ItemEnum, gamma: &mut Gamma) -> Vec<Item> {
    // Create a trait
    let consumers = gamma.get_enum_consumers(enum_);
    let trait_methods: Vec<TraitItemMethod> = Vec::from_iter(
        consumers
            .iter()
            .map(|consumer| transform_consumer_fn_to_trait_item(&consumer, gamma)),
    );

    let mut trait_ = create_trait(
        &enum_.ident,
        &trait_methods
            .iter()
            .zip(consumers.iter())
            .map(|(method, consumer)| {
                TraitItem::Method(TraitItemMethod {
                    // For each consumer if there is no match statement, add a default impl to the
                    // trait
                    default: if get_consumer_match_statement(&consumer).is_ok() {
                        None
                    } else {
                        // If the return type is the trait, we cannot use the default impl
                        let return_type = consumer.sig.output.get_delta_type(None);
                        if return_type.is_some() && return_type.unwrap().name == enum_.ident {
                            None
                        } else {
                            if let Expr::Block(block) = transform_consumer_expr(
                                    &Expr::Block(ExprBlock{
                                        block: *consumer.block.clone(),
                                        attrs: Vec::new(),
                                        label: None,
                                    }),
                                    get_fn_arg_name(&consumer.sig.inputs.first().unwrap()),
                                    Vec::new(),
                                    &gamma,
                            ) {
                                Some(block.block)
                            } else {
                                panic!("This should always be a block!")
                            }
                        }
                    },
                    ..method.clone()
                })
            })
            .collect::<Vec<TraitItem>>(),
        enum_.vis.clone(),
    );
    gamma.add_trait(&trait_);

    let mut items = vec![Item::Trait(trait_.clone())];

    // For each variant of the enum create a struct and an impl
    for variant in enum_.variants.iter() {
        // Create the struct
        let struct_ = create_struct(
            &variant.ident,
            &enum_.ident,
            transform_type_struct_fields(&variant.fields.clone(), |type_: Type| transform_type_fp(type_, gamma)),
            enum_.vis.clone(),
        );
        println!("Adding {} struct to gamma", struct_.ident);
        gamma.add_struct(&struct_);
        items.push(Item::Struct(struct_.clone()));

        // Add emtpy generator to gamma
        let impl_ = create_impl(&enum_.ident, &variant.ident, Vec::new());
        gamma.add_generator(&trait_, &struct_, &impl_);

        // Collect methods
        // TODO handle trait method
        let impl_items =
            consumers
                .iter()
                .zip(trait_methods.iter())
                .filter_map(|(consumer, trait_method)| {
                    // Get the expr for the new destructor
                    let consumer_expr: Option<Expr> =
                        match get_match_expr_for_enum(consumer, &variant.ident) {
                            // If there is an arm in the match statement, we can use it
                            Ok(expr) => Some(expr),
                            // Otherwise we will have to use the method body for all the cases
                            Err(e) => {
                                // 1. The trait has a default impl for this method. This is only possible if the
                                //    return type of the consumer is not the same as the enum.
                                //    I.e. it is not possible to have a defualt implementation with sig:
                                //      fn foo() -> Self / fn foo() -> Box<Self>
                                //    In this case we can just use the default impl so no impl is needed here
                                let return_type = consumer.sig.output.get_delta_type(None);
                                if return_type.is_some() && return_type.unwrap().name != enum_.ident
                                {
                                    return None;
                                }

                                // 2. Same as above but the return type is the same as the enum. In this case we
                                //    need to copy the default impl each time.
                                //    TODO create a function and use it here instead of duplicate code
                                Some(Expr::Block(ExprBlock {
                                    block: *consumer.block.clone(),
                                    attrs: Vec::new(),
                                    label: None,
                                }))
                            }
                        };

                    // If we dont get an expr for this destructor then we can skip it. (This will be
                    // because the expr is already defined in the default impl of the trait)
                    if consumer_expr.is_none() {
                        return None;
                    }

                    let expr = transform_consumer_expr(
                        &consumer_expr.unwrap(),
                        get_fn_arg_name(&consumer.sig.inputs.first().unwrap()),
                        Vec::from_iter(
                            variant
                                .fields
                                .iter()
                                .map(|field| field.ident.clone().unwrap()),
                        ),
                        gamma,
                    );

                    Some(ImplItem::Method(create_impl_method(
                        &trait_method.sig,
                        // If the expr is already a block take its block
                        &if let Expr::Block(expr_block) = expr {
                            expr_block.block
                        // Otherwise create a block with the single expr
                        } else {
                            Block {
                                brace_token: token::Brace::default(),
                                stmts: vec![Stmt::Expr(expr)],
                            }
                        },
                    )))
                });

        // Create the impl
        let impl_ = create_impl(&enum_.ident, &variant.ident, Vec::from_iter(impl_items));
        // Update gamma with real impl
        gamma.add_generator(&trait_, &struct_, &impl_);
        items.push(Item::Impl(impl_));
    }

    return items;
}

/// Transforms a destructor of a trait into a consumer of the enum
///
/// * `trait_` - The trait that the destructor belongs to
/// * `destructor` - The destructor to transform
/// * `enum_ident` - The ident of the enum that the new generator should be created for
/// * `gamma` - The gamma context
fn transform_destructor(
    trait_: &ItemTrait,
    destructor: &TraitItemMethod,
    enum_: &ItemEnum,
    gamma: &mut Gamma,
) -> Item {
    // Collect all the generics from all the implementations of the trait destructor
    let mut generics = trait_.generics.clone();
    let enum_generics = trait_.generics.clone();
    for (_, generator_impl) in gamma.get_generators(&trait_.ident) {
        // For the implementation find the generics for the trait
        let trait_generics = get_generics_from_path_segment(
            &*generator_impl.trait_.unwrap().1.segments.first().unwrap(),
        );

        // For the implementation find all the generics
        let impl_generics = generator_impl.generics;

        // Remove the generics that are from the trait from all the generics of the implementation
        let struct_generics: Vec<GenericParam> = Vec::from_iter(
            impl_generics
                .params
                .iter()
                .filter(|param| {
                    !trait_generics
                        .params
                        .iter()
                        .any(|trait_param| trait_param == *param)
                })
                .cloned(),
        );

        // Add the generics that a just for the struct (not from the trait) to the list of generics
        generics.params.extend(struct_generics);
    }

    let (mut signature, enum_instance_name) = transform_destructor_signature(
        &destructor.sig,
        &enum_.ident,
        &generics,
        &enum_generics,
        gamma,
    );

    let mut arms: Vec<syn::Arm> = Vec::new();
    // If any of the impl do not have an implementation of the destructor then we need to create a
    // wildcard argument
    let mut wild_card_arm_required = false;
    for (generator, generator_impl) in gamma.get_generators(&trait_.ident).iter() {
        let result = transform_destructor_impl(
            generator,
            destructor,
            &enum_.ident,
            &enum_instance_name,
            generator_impl,
            gamma,
        );

        match result {
            Ok(arm) => {
                arms.push(arm);
            }
            Err(NotFound { .. }) => wild_card_arm_required = true,
        }
    }

    // If required, add the wild card arm
    if wild_card_arm_required {
        // Get impl in the trait
        let mut body = Expr::Block(ExprBlock {
            block: Gamma::get_destructor_impl_for_trait(trait_, &destructor.sig.ident)
                .unwrap()
                .default
                .unwrap(),
            attrs: Vec::new(),
            label: None,
        });

        // TODO Currently this Vec::new() means mutable things cannot have a wild card arm. Fix by
        // NOTE trait_ident here is wrong/irrelevant
        body = transform_destructor_expr(
            &body,
            Vec::new(),
            &enum_instance_name,
        );

        // Create wild card arm with this body
        arms.push(ast::create::create_wildcard_match_arm(body));
    }

    let mut match_expr = ast::create::create_match_statement(&enum_instance_name, arms);

    if is_dyn_box_generator_return(&signature, gamma) {
        // TODO change return type
        signature = Signature {
            output: transform_dyn_box_destructor_signature_output(&signature.output),
            ..signature
        };

        // TODO Change every return statement
        let mut rdbdrs = ReplaceDynBoxDestructorReturnStatements;
        rdbdrs.visit_expr_mut(&mut match_expr);
    }

    // TODO for now all functions are public -> check if the trait is public
    let func =
        ast::create::create_function(signature, vec![Stmt::Expr(match_expr)], trait_.vis.clone());

    gamma.add_enum_consumer(&enum_, &func);
    Item::Fn(func)
}

pub fn transform_dyn_box_destructor_signature_output(output: &ReturnType) -> ReturnType {
    if let ReturnType::Type(_, type_) = output {
        return create_return_type_from_ident(&type_.get_delta_type().name);
    }
    panic!("Unsupported return type for destructor");
}

/// Transform a function implementation of a destructor into an arm of the consumers match
/// statement
///
/// * `generator` - The generator that the destructor belongs to
/// * `destructor` - The destructor that the impl is of
/// * `enum_name` - The name of the enum that the match arm should be created for
/// * `enum_instance_name` - The name of the instance of the enum
/// * `impl_` - The implementation of the generator
fn transform_destructor_impl(
    generator: &ItemStruct,
    destructor: &TraitItemMethod,
    enum_name: &Ident,
    enum_instance_name: &Ident,
    impl_: &ItemImpl,
    gamma: &Gamma,
) -> std::result::Result<Arm, NotFound> {
    // Find the implementation of the method
    let method_result = Gamma::get_destructor_impl_for_generator(&impl_, &destructor.sig.ident);
    if method_result.is_err() {
        return Err(method_result.err().unwrap());
    }

    let mut block: Block = method_result.unwrap().block;

    // The name of the varaibles created in the below let expressions
    let mut self_mutable_fields = Vec::new();
    // let mut new_delta = new_delta.clone();
    // If the method is mutable self
    if is_mutable_self(&destructor.sig) {
        // Create a new mut variable for each attribute in the struct equal to the value in the struct
        // Eg for circle
        // let mut radius = self.radius
        // This isnt in delta
        // new_self_mut_field = Some(Ident::new("new", Span::call_site()));

        // let local = create_let_stmt(
        //     new_self_mut_field.as_ref().unwrap(),
        //     &create_expr_from_ident(&enum_instance_name),
        //     true
        // );
        // new_delta.collect_for_local(&local, gamma);
        // block = add_stmts_to_block(
        //     &Stmt::Local(
        //         local,
        //     ),
        //     &block,
        //     0
        // );
        for field in generator.fields.iter() {
            let field_name = field.ident.clone().unwrap();
            self_mutable_fields.push(field_name.clone());
        }

        // TODO Add in return statement
        block = add_stmts_to_block(
            &Stmt::Expr(Expr::Struct(ExprStruct {
                attrs: Vec::new(),
                path: create_path_from_ident(&generator.ident),
                brace_token: token::Brace::default(),
                dot2_token: None,
                fields: Punctuated::from_iter(self_mutable_fields.iter().map(|field| FieldValue {
                    attrs: Vec::new(),
                    member: Member::Named(field.clone()),
                    colon_token: None,
                    expr: Expr::Path(create_expr_path_from_path(create_path_from_ident(&field))),
                })),
                rest: None,
            })),
            &block,
            block.stmts.len(),
        );
    }

    let mut expr = Expr::Block(ExprBlock {
        block,
        attrs: Vec::new(),
        label: None,
    });

    // TODO Maybe problaly clone it?
    // TODO Then for every *radius = something remove the deref => radius = something
    let mut rsfa = ReplaceSelfFieldAssignments {
        self_fields: generator
            .fields
            .iter()
            .map(|field| field.ident.clone().unwrap())
            .collect(),
    };
    rsfa.visit_expr_mut(&mut expr);

    // Then return a new instance of the type with these mut variables
    // Transform the body of the method
    expr = transform_destructor_expr(
        &expr,
        self_mutable_fields,
        enum_instance_name,
    );

    // Create the arm of the match statement
    let path = ast::create::create_path_for_enum(enum_name, &generator.ident);
    Ok(ast::create::create_match_arm(
        path,
        get_struct_attrs(&generator),
        expr,
        is_mutable_self(&destructor.sig),
    ))
}

/// Given expression for destructor covert all method calls
///
/// Replace all method calls to the destructor with the corresponding consumer function call
///
/// * `expr` - The expression to transforms
/// * `old_delta` - The delta that the method call is from
/// * `new_delta` - The delta that the method call is to
/// * `self_mutable_fields` - If this is a mutable self destructor, then all the fields in the
/// struct are added in here. These are handled seperately in the transform (not deferencced)
/// * `gamma` - The gamma that the method call is in
fn transform_destructor_expr(
    expr: &Expr,
    self_mutable_fields: Vec<Ident>,
    enum_name: &Ident,
) -> Expr {
    // TODO replace this logic with standard transform_expr
    let mut expr_clone = expr.clone();

    let mut rfc = ReplaceFieldCalls {
        self_mut_fields: self_mutable_fields,
    };
    rfc.visit_expr_mut(&mut expr_clone);

    let mut rs = ReplaceSelf {
        enum_name: enum_name.clone(),
    };
    rs.visit_expr_mut(&mut expr_clone);
    return expr_clone;
}

fn transform_consumer_expr(
    expr: &Expr,
    self_arg_name: Ident,
    trait_attributes: Vec<Ident>,
    gamma: &Gamma,
) -> Expr {
    let mut expr_clone = expr.clone();
    let mut tc = TransformConsumer {
        trait_attributes,
        gamma: gamma.clone(),
        self_arg_name,
    };
    tc.visit_expr_mut(&mut expr_clone);
    expr_clone
}

/// Convert signature of destructor to consumer signature
///
/// Replace &self with Box<T> and replace self with T
///
/// * `signature` - The signature of the trait method
/// * `enum_name` - The name of the enum (interface) which replaces self
/// * `gamma` - Gamma
///
/// Returns:
/// - the function signature and the name of the type which replaces self if self is present
/// - the name of the argument which replaces self if self is present
fn transform_destructor_signature(
    signature: &Signature,
    enum_name: &Ident,
    generics: &Generics,
    enum_generics: &Generics,
    gamma: &mut Gamma,
) -> (Signature, Ident) {
    let enum_instance_name = transform_type_to_name(enum_name);

    // Transform arguments
    let new_inputs = syn::punctuated::Punctuated::from_iter(signature.inputs.iter().map(|item| {
        let create_self_consumer_signature = |as_ref| {
            // Add generics to enum_name
            create_consumer_signature_arg(enum_name, &enum_instance_name, as_ref, &enum_generics)
        };

        // Replace self with enum
        if let syn::FnArg::Receiver(Receiver { mutability, .. }) = item {
            // TODO use borrow as required and check this is acutally self
            return create_self_consumer_signature(mutability.is_none());
        }
        if let syn::FnArg::Typed(PatType { pat, ty, .. }) = item {
            if let Pat::Ident(pat_ident) = &**pat {
                // The type of the thing
                let arg_type = ty.get_delta_type();

                // Check if the type is in the geneators
                if gamma.is_interface(&arg_type.name) {
                    // If self
                    if arg_type.name == "Self" || pat_ident.ident == "self" {
                        return create_self_consumer_signature(false);
                    }
                    return create_consumer_signature_arg(
                        &arg_type.name,
                        &pat_ident.ident,
                        false,
                        &enum_generics,
                    );
                }
            }
        }
        item.clone()
    }));

    // NOTE output transfomration is handled later expect for handling returning self for mut self destructors

    let mut output = signature.output.clone();

    if is_mutable_self(&signature) {
        if !matches!(signature.output, ReturnType::Default) {
            panic!("Transforming mutable destructors without outputs not supported");
        }
        output = ReturnType::Type(
            token::RArrow::default(),
            Box::new(Type::Path(TypePath {
                qself: None,
                path: create_path_from_ident(enum_name),
            })),
        );
    }

    let sig = syn::Signature {
        inputs: new_inputs,
        generics: generics.clone(),
        output,
        ..signature.clone()
    };
    gamma.set_signature(&sig.ident, &sig);

    (
        sig,
        enum_instance_name.clone(),
    )
}

pub fn transform_consumer_signature(signature: &Signature, gamma: &mut Gamma) -> Signature {
    let mut inputs = signature.inputs.clone();

    // Get the self arg
    let consumer_arg: FnArg = inputs.iter().next().unwrap().clone();
    let self_type = consumer_arg.get_delta_type(None);

    // Ignoring the first element transfrom each argument
    let mut new_inputs = Vec::from_iter(inputs.iter().skip(1).map(|arg| {
        // TODO make all args with type of enum, Box<dyn T>
        let type_ = arg.get_delta_type(None);
        if gamma.is_enum(&type_.name) && matches!(type_.ref_type, RefType::Box(_) | RefType::None)
        {
            // Create box dyn of fn arg
            return create_dyn_box_arg(&arg);
        }
        arg.clone()
    }));
    new_inputs.insert(
        0,
        create_self_fn_arg(if matches!(consumer_arg.get_ref_type(), RefType::Ref(_)) {
            consumer_arg.get_ref_type()
        } else {
            RefType::Box(Box::new(RefType::None))
        }),
    );

    let mut output = signature.output.clone();

    if let ReturnType::Type(ra, ty) = &output {
        if gamma.is_enum(&ty.get_delta_type().name) {
            // Create box dyn of fn arg
            output = ReturnType::Type(*ra, Box::new(create_dyn_box_of_type(&ty)))
        }
    }

    let sig = Signature {
        inputs: syn::punctuated::Punctuated::from_iter(new_inputs),
        output,
        ..signature.clone()
    };
    gamma.set_signature(&sig.ident, &sig);

    sig
}

/// Given the name of a type get a sensible name for the object
///
/// * `type_ident` - The name of the type e.g. Foo
///
/// # Examples
///
/// ```
/// use syn::Ident;
/// use syn::__private::Span;
/// use rfood::transform::transformer::transform_type_to_name;
///
/// let type_ = Ident::new("Something", Span::call_site());
/// assert_eq!(transform_type_to_name(&type_).to_string(), "something");
/// ```
pub fn transform_type_to_name(type_ident: &Ident) -> Ident {
    Ident::new(&type_ident.to_string().to_lowercase(), type_ident.span())
}

fn transform_struct_instantiation_path_for_enum(
    expr_struct: &ExprStruct,
    gamma: &Gamma,
    delta: &Delta,
) -> Path {
    // Get the name of the enum
    let datatype_name = gamma
        .get_enum_variant_enum(&expr_struct.path.get_delta_type().name)
        .unwrap();
    // Add the enum in front of the struct
    let mut new_path_vec = vec![PathSegment {
        ident: datatype_name.ident.clone(),
        arguments: PathArguments::None,
    }];
    new_path_vec.append(&mut Vec::from_iter(
        expr_struct.path.segments.clone().iter().cloned(),
    ));

    // Add it to the front of the struct path segments
    Path {
        segments: Punctuated::from_iter(new_path_vec),
        ..expr_struct.path.clone()
    }
}

fn transform_expr_type(
    expr: &Expr,
    current_type: &DeltaType,
    required_type: &EType,
    gamma: &Gamma,
) -> Expr {
    println!("Transforming expr type for {:?} to {:?}", current_type, required_type);

    // If the current type equivalent to the required type
    if let EType::DeltaType(required_type) = required_type {
        if current_type.is_equaivalent(&required_type, &gamma) {
            return expr.clone();
        }
    }

    match &required_type {
        EType::Any | EType::None => expr.clone(),
        EType::RefType(rt) | &EType::DeltaType(DeltaType{ref_type: rt, ..}) => match (&current_type.ref_type, rt){
            (left, right) if left == right => expr.clone(),
            // Box -> None / Ref -> None
            (RefType::Box(box inner) | RefType::Ref(box inner), RefType::None) => create_dereference_of_expr(
                &transform_expr_type(
                    expr,
                    &DeltaType{name: current_type.name.clone(), ref_type: inner.clone()},
                    &EType::RefType(RefType::None),
                    &gamma
                )
            ),
            // None -> Box
            (RefType::None, RefType::Box(box inner)) => create_box_of_expr(
                &transform_expr_type(
                    expr,
                    &DeltaType{name: current_type.name.clone(), ref_type: RefType::None},
                    &EType::RefType(inner.clone()),
                    &gamma
                )
            ),
            // None -> Ref
            (RefType::None, RefType::Ref(box inner)) => create_reference_of_expr(
                &transform_expr_type(
                    expr,
                    &DeltaType{name: current_type.name.clone(), ref_type: RefType::None},
                    &EType::RefType(inner.clone()),
                    &gamma
                )
            ),
            // Box -> Ref / Ref -> Box
            (RefType::Box(box current_inner), RefType::Ref(box required_inner)) | (RefType::Ref(box current_inner), RefType::Box(box required_inner)) => {
                create_reference_of_expr(&create_dereference_of_expr(
                    &transform_expr_type(
                        expr,
                        &DeltaType{name: current_type.name.clone(), ref_type: current_inner.clone()},
                        &EType::RefType(required_inner.clone()),
                        &gamma
                    )
                ))
            }
            _ => panic!("Cannot transform {:?} to {:?}", current_type, required_type),
        }
    }
}

fn transform_block(
    block: &Block,
    transform_type: &TransformType,
    gamma: &Gamma,
    delta: &Delta,
    return_type: EType,
) -> Block {
    let mut delta = delta.clone();
    let stmts = Vec::from_iter(block.stmts.iter().enumerate().map(|(index, stmt)| {
        transform_statement(
            &stmt,
            transform_type,
            gamma,
            &mut delta,
            if index == block.stmts.len() - 1 {
                return_type.clone()
            } else {
                EType::None
            },
        )
    }));
    Block {
        stmts,
        ..block.clone()
    }
}

fn transform_expr_inner(
    expr: &Expr,
    transform_type: &TransformType,
    gamma: &Gamma,
    delta: &mut Delta,
    return_type: EType,
) -> Expr {
    match (transform_type, expr) {
        (_, Expr::Unary(_) | Expr::Path(_)) => {
            // Remove any existing derefs so we can fix the type manually
            // if let EType::DeltaType(delta_type) = return_type {
            //     let expr = clean_type(expr);
            //     let current_type = delta.get_type_of_expr(&expr, gamma);
            //     if let Ok(current_type) = current_type {
            //         return transform_expr_type(&expr, &current_type, &delta_type, gamma);
            //     }
            // }
            return expr.clone();
        },
        (TransformType::OOPToFP, Expr::MethodCall(expr_method_call))
            if gamma.is_consumer(&expr_method_call.method) =>
        {
            println!("Transforming expr method call");

            let ExprMethodCall {
                receiver,
                method,
                args,
                ..
            } = expr_method_call;

            let mut new_args = vec![*receiver.clone()];
            let old_args: Vec<Expr> = args.iter().cloned().collect();
            new_args.extend(old_args);
            let mut fn_expr = create_function_call(&method, Punctuated::from_iter(new_args));

            // Perform regular transform on function call
            fn_expr = transform_expr(&fn_expr, &transform_type, &gamma, &delta, return_type.clone());

            // If the method is a mutable self call
            if gamma.is_mutable_self_method_call(&expr_method_call, &delta) {
                // Overwrite the receiver
                Expr::Assign(create_assignment_expr(*receiver.clone(), fn_expr.clone()))
            } else {
                fn_expr
            }
        }
        // Any other method call, transform all the args and the receiver
        (_, Expr::MethodCall(method_call)) => {
            // Get the signature of the method call, NOTE this will fail if any method call are
            // made which are not on destructors TODO fix
            let reciever_type = delta
                .get_type_of_expr(&method_call.receiver, &gamma)
                .unwrap();
            let signature = gamma.get_signature(&method_call.method);
            
            println!("Transforming method call {}", signature.as_ref().unwrap().ident);
            println!("Delta is {:?}", delta);

            Expr::MethodCall(ExprMethodCall {
                receiver: Box::new(transform_expr(
                    &method_call.receiver,
                    transform_type,
                    gamma,
                    &delta,
                    if matches!(signature.as_ref().unwrap().inputs.first().unwrap().get_delta_type(Some(reciever_type.name.clone())).ref_type, RefType::Box(_)) {
                        EType::RefType(RefType::Box(Box::new(RefType::None)))
                    } else {
                        EType::Any
                    }
                )),
                // Skip one to skip the receiver argument
                args: Punctuated::from_iter(method_call.args.iter().enumerate().map(
                    |(index, arg)| {
                        println!("Transforming type of arg to {:?}", signature.as_ref().unwrap().inputs[index + 1].get_delta_type(Some(reciever_type.name.clone())));
                        transform_expr(
                            arg,
                            transform_type,
                            gamma,
                            &delta,
                            match &signature {
                                Ok(signature) => EType::DeltaType(
                                    signature.inputs[index + 1]
                                        .get_delta_type(Some(reciever_type.name.clone())),
                                ),
                                Err(_) => EType::Any,
                            },
                        )
                    },
                )),
                ..method_call.clone()
            })
        }
        // If the experssion is calling a consumer and we are transforming from FP to OOP
        // Then we should replace the call with a method call
        (TransformType::FPToOOP, Expr::Call(expr_call))
            if gamma.is_destructor(&get_function_call_name(expr_call)) =>
        {
            // Extract the first argument to the function
            let mut args = expr_call.args.clone();
            let first_arg = clean_type(&args.pop_first().unwrap());

            // Create method call
            let method_call =
                create_method_call(&get_function_call_name(expr_call), &first_arg, &args);
            // Performance regular transformations to the new method call (fix typing of args)
            transform_expr(&method_call, &transform_type, &gamma, &delta, return_type.clone())
        }
        (_, Expr::Call(expr_call)) if new_box_call_expr(expr).is_ok() => {
            // Get the inner extression of the box call
            let mut inner_expr = new_box_call_expr(expr).unwrap();

            // Transform the inner expression
            inner_expr = transform_expr(
                &inner_expr,
                &transform_type,
                &gamma,
                &delta,
                return_type.clone(),
            );

            // If the expected return type is not a box or the inner type is a box, then just
            // return the inner expression
            if let EType::DeltaType(dt) = &return_type {
                if !matches!(dt.ref_type, RefType::Box(_))
                    || matches!(
                        delta.get_type_of_expr(&inner_expr, gamma).unwrap().ref_type,
                        RefType::Box(_)
                    )
                {
                    return inner_expr;
                }
            }

            // Otherwise recreate a box of the inner expression
            create_box_of_expr(&inner_expr)
        }
        (_, Expr::Call(expr_call)) => {
            if let ExprCall {
                func: box Expr::Path(ExprPath { path, .. }),
                ..
            } = expr_call
            {
                let signature = gamma
                    .get_signature(&get_function_call_name(expr_call))
                    .unwrap();

                return Expr::Call(ExprCall {
                    func: Box::new(match &*expr_call.func {
                        Expr::Path(_) => *expr_call.func.clone(),
                        _ => transform_expr(
                                &expr_call.func,
                                transform_type,
                                gamma,
                                &delta,
                                EType::Any,
                        ),
                    }),
                    args: Punctuated::from_iter(expr_call.args.iter().enumerate().map(
                        |(index, arg)| {
                            transform_expr(
                                // Remove existing typing from fn arg
                                &clean_type(arg),
                                transform_type,
                                gamma,
                                &delta,
                                EType::DeltaType(signature.inputs[index].get_delta_type(None)),
                            )
                        },
                    )),
                    ..expr_call.clone()
                });
            }
            panic!("Cannot transform non path calls")
        }
        (_, Expr::Return(expr_return)) if expr_return.expr.is_some() => {
            Expr::Return(ExprReturn {
                expr: Some(Box::new(transform_expr(
                    &expr_return.clone().expr.unwrap(),
                    transform_type,
                    gamma,
                    &delta,
                    return_type.clone(),
                ))),
                ..expr_return.clone()
            }
        )},
        (TransformType::OOPToFP, Expr::Struct(expr_struct))
            if gamma.is_enum_or_variant(&expr_struct.path.get_delta_type().name) =>
        {
            let struct_ = Expr::Struct(ExprStruct {
                path: transform_struct_instantiation_path_for_enum(expr_struct, gamma, &delta),
                fields: Punctuated::from_iter(expr_struct.fields.iter().map(|field| {
                    // Get the enum
                    let enum_variant_ident = expr_struct.path.get_delta_type().name;
                    let enum_variant =
                        gamma.get_enum_variant(&enum_variant_ident, &enum_variant_ident);
                    let mut enum_delta = Delta::new();
                    enum_delta.collect_for_enum_variant(&enum_variant.unwrap(), false);

                    let required_type = enum_delta.get_type_of_member(&field.member);
                    let new_expr = transform_expr(
                        &field.expr,
                        transform_type,
                        gamma,
                        &delta,
                        EType::DeltaType(required_type.clone()),
                    );

                    // Check type of expr matches required type
                    // let new_expr_type = delta.get_type_of_expr(&new_expr, gamma).unwrap();

                    FieldValue {
                        expr: new_expr,
                        // expr: transform_expr_type(
                        //     &new_expr,
                        //     &new_expr_type,
                        //     &required_type,
                        //     &gamma,
                        // ),
                        ..field.clone()
                    }
                })),
                ..expr_struct.clone()
            });
            // if let EType::DeltaType(dt) = return_type {
            //     return transform_expr_type(
            //         &struct_,
            //         &delta.get_type_of_expr(&struct_, gamma).unwrap(),
            //         &dt,
            //         &gamma,
            //     );
            // }
            return struct_;
        }
        (TransformType::FPToOOP, Expr::Struct(expr_struct))
            if gamma.get_generator_trait(&expr_struct.path.segments.last().unwrap().ident)
                .is_some() =>
        {
            let struct_ = Expr::Struct(ExprStruct {
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter(vec![expr_struct
                        .path
                        .segments
                        .last()
                        .unwrap()
                        .clone()]),
                },
                fields: Punctuated::from_iter(expr_struct.fields.iter().map(|field| {
                    // Get the enum
                    let struct_ident = get_path_call_name(&expr_struct.path);
                    let struct_ = gamma.get_struct_by_name(&struct_ident);
                    let mut struct_delta = Delta::new();
                    // TODO check this...
                    struct_delta.collect_for_struct(&struct_, RefType::None);

                    let required_type = struct_delta.get_type_of_member(&field.member);
                    let new_expr = transform_expr(
                        &field.expr,
                        transform_type,
                        gamma,
                        &delta,
                        EType::DeltaType(required_type.clone()),
                    );

                    // Check type of expr matches required type
                    // let new_expr_type = delta.get_type_of_expr(&new_expr, gamma).unwrap();

                    FieldValue {
                        // TODO Move this transform type into transform expr
                        expr: new_expr,
                        // expr: transform_expr_type(
                        //     &new_expr,
                        //     &new_expr_type,
                        //     &required_type,
                        //     &gamma,
                        // ),
                        ..field.clone()
                    }
                })),
                ..expr_struct.clone()
            });
            // if let EType::DeltaType(dt) = return_type {
            //     return transform_expr_type(
            //         &struct_,
            //         &delta.get_type_of_expr(&struct_, gamma).unwrap(),
            //         &dt,
            //         &gamma,
            //     );
            // }
            return struct_;
        }
        (_, Expr::Block(expr_block)) => Expr::Block(ExprBlock {
            block: transform_block(
                &expr_block.block,
                transform_type,
                gamma,
                &delta,
                return_type.clone(),
            ),
            ..expr_block.clone()
        }),
        (_, Expr::Match(expr_match)) => {
            println!("Transforming expr match");
            println!("Transforming pat");
            let e1 = Box::new(transform_expr(
                    &*expr_match.expr,
                    transform_type,
                    gamma,
                    &delta,
                    EType::RefType(RefType::Ref(Box::new(RefType::None))),
                ));
            println!("Transformed pat, e1: {:?}", e1);
            let e = Expr::Match(ExprMatch {
                // Transform the match epxr,
                // expr: Box::new(transform_expr(
                //     &*expr_match.expr,
                //     transform_type,
                //     gamma,
                //     &delta,
                //     EType::Any,
                // )),
                expr: e1,
                // Transform the body of the match with the context of the struct (all borrows)
                arms: expr_match
                    .arms
                    .iter()
                    .map(|arm| {
                        // If this is a match expr over a struct (TODO this probably also has to
                        // happen for enums)
                        // Then each value collected is a borrow
                        // TODO
                        delta.collect_for_arm(&arm, &gamma);
                        Arm {
                            body: Box::new(transform_expr(
                                &arm.body,
                                transform_type,
                                gamma,
                                &delta,
                                return_type.clone(),
                            )),
                            ..arm.clone()
                        }
                    })
                    .collect(),
                ..expr_match.clone()
            });
            println!("Done Transforming expr match");
            e
        }
        (_, Expr::Macro(expr_macro)) => {
            // Try and parse the macros parameters into expressions
            let parser = Punctuated::<Expr, Token![,]>::parse_separated_nonempty;
            let params = parser.parse2(expr_macro.mac.tokens.clone());

            if params.is_err() {
                panic!("Could not parse macro params, only macros with expr params are supported");
            }

            let params = params.unwrap();
            // Transform the parameter expressions
            let params: Punctuated<Expr, Token![,]> = params
                .iter()
                .map(|param| {
                    transform_expr(param, transform_type, gamma, &delta, return_type.clone())
                })
                .collect();

            Expr::Macro(ExprMacro {
                mac: Macro {
                    tokens: quote!(#params),
                    ..expr_macro.mac.clone()
                },
                ..expr_macro.clone()
            })
        }
        (_, Expr::Binary(expr_binary)) => {
            let new_left_expr = transform_expr(
                &*expr_binary.left,
                transform_type,
                gamma,
                &delta,
                EType::Any,
            );
            let new_left_expr_type = delta.get_type_of_expr(&new_left_expr, gamma).unwrap();

            Expr::Binary(ExprBinary{
                left: Box::new(new_left_expr),
                right: Box::new(
                    transform_expr(
                        &*expr_binary.right,
                        transform_type,
                        gamma,
                        &delta,
                        EType::DeltaType(new_left_expr_type.clone()),
                    )
                ),
                ..expr_binary.clone()
            })
        },
        (_, Expr::Reference(expr_ref)) => {
            if let EType::RefType(ref_type) | EType::DeltaType(DeltaType{ref_type, ..}) = &return_type {
                if let RefType::Ref(inner_ref_type) = ref_type {
                    Expr::Reference(ExprReference{
                        expr: Box::new(transform_expr(
                            &expr_ref.expr,
                            transform_type,
                            gamma,
                            &delta,
                            EType::RefType(*inner_ref_type.clone())
                        )),
                        ..expr_ref.clone()
                    })
                } else {
                    transform_expr(
                        &expr_ref.expr,
                        transform_type,
                        gamma,
                        &delta,
                        return_type,
                    )
                }
            } else {
                *expr_ref.expr.clone()
            }
        }
        (_, Expr::If(expr_if)) => {
            Expr::If(ExprIf{
                cond: Box::new(
                    transform_expr(&*expr_if.cond, transform_type, gamma, &delta, EType::DeltaType(DeltaType::new("bool", RefType::None)))
                ),
                then_branch: transform_block(
                    &expr_if.then_branch,
                    transform_type,
                    gamma,
                    &delta,
                    return_type.clone(),
                ),
                else_branch: if let Some((else_token, box else_branch)) = expr_if.else_branch.clone() {
                    Some((
                        else_token,
                        Box::new(transform_expr(
                            &else_branch,
                            transform_type,
                            gamma,
                            &delta,
                            return_type.clone(),
                        ))
                    ))
                } else {
                    None
                },
                ..expr_if.clone()
            })
        },
        (_, Expr::Paren(expr_paren)) => {
            Expr::Paren(ExprParen{
                expr: Box::new(transform_expr(
                    &expr_paren.expr,
                    transform_type,
                    gamma,
                    &delta,
                    return_type.clone(),
                )),
                ..expr_paren.clone()
            })
        },
        _ => {
            println!("Skipping unsupported {:?} with delta {:?}", expr, delta);
            expr.clone()
        }
    }
}

fn transform_expr(
    expr: &Expr,
    transform_type: &TransformType,
    gamma: &Gamma,
    delta: &Delta,
    return_type: EType,
) -> Expr {
    // Clone the delta at this stage
    let mut delta = delta.clone();

    // Transform the expr
    let expr = clean_type(&transform_expr_inner(expr, transform_type, gamma, &mut delta, return_type.clone()));

    // Transform the expression type
    let expr_type = delta.get_type_of_expr(&expr, gamma);
    match expr_type {
        Ok(et) => {
            transform_expr_type(&expr, &et, &return_type, gamma)
        },
        _ => expr
    }
    // expr.clone()
}

fn transform_statement(
    statement: &Stmt,
    transform_type: &TransformType,
    gamma: &Gamma,
    delta: &mut Delta,
    return_type: EType,
) -> Stmt {
    match statement {
        Stmt::Local(local) => {
            let init_unwrap = local.init.as_ref().unwrap();
            let trans_local = Local {
                init: Some((
                    init_unwrap.0,
                    Box::new(transform_expr(
                        &init_unwrap.1,
                        transform_type,
                        &gamma,
                        delta,
                        EType::Any,
                    )),
                )),
                ..local.clone()
            };
            delta.collect_for_local(&trans_local, gamma);
            Stmt::Local(trans_local)
        },
        Stmt::Semi(expr, semi) => {
            Stmt::Semi(
                transform_expr(&expr, transform_type, gamma, delta, return_type),
                *semi,
            )
        }
        Stmt::Expr(expr) => Stmt::Expr(transform_expr(
            &expr,
            transform_type,
            gamma,
            delta,
            return_type,
        )),
        _ => panic!("Unsupported statement {:?}", statement),
    }
}

pub fn transform_item(
    item: &syn::Item,
    transform_type: &TransformType,
    gamma: &Gamma,
    delta: &mut Delta,
) -> syn::Item {
    match item {
        Item::Fn(item_fn) => Item::Fn(transform_function(item_fn, transform_type, gamma, delta)),
        Item::Impl(item_impl) => {
            let for_type = item_impl.self_ty.get_delta_type().name;
            Item::Impl(ItemImpl {
                items: item_impl
                    .items
                    .iter()
                    .map(|item| transform_impl_item(item, &for_type, transform_type, gamma, delta))
                    .collect(),
                ..item_impl.clone()
            })
        },
        Item::Const(item_cost) => {
            delta.collect_for_const(&item_cost);
            item.clone()
        },
        _ => item.clone(),
    }
}

pub fn transform_impl_item(
    impl_item: &syn::ImplItem,
    impl_for_type: &Ident,
    transform_type: &TransformType,
    gamma: &Gamma,
    delta: &Delta,
) -> syn::ImplItem {
    let mut delta = delta.clone(); 
    match impl_item {
        ImplItem::Method(impl_item_method) => {
            let return_type = impl_item_method
                .sig
                .output
                .get_delta_type(Some(impl_for_type.clone()));
            let block_return_type = match return_type {
                Some(rt) => EType::DeltaType(rt),
                None => EType::None,
            };

            ImplItem::Method(ImplItemMethod {
                block: {
                    delta.collect_for_sig(&impl_item_method.sig, Some(impl_for_type));
                    transform_block(
                        &impl_item_method.block,
                        transform_type,
                        gamma,
                        &delta,
                        block_return_type,
                    )
                },
                ..impl_item_method.clone()
            })
        }
        _ => {
            // println!("Skipping unsupported {:?}", impl_item);
            impl_item.clone()
        }
    }
}

/// Transform all the statements in a fuction
fn transform_function(func: &ItemFn, transform_type: &TransformType, gamma: &Gamma, delta: &Delta) -> syn::ItemFn {
    let mut delta = delta.clone();
    delta.collect_for_sig(&func.sig, None);

    let return_type = func.sig.output.get_delta_type(None);
    let block_return_type = match return_type {
        Some(rt) => EType::DeltaType(rt),
        None => EType::None,
    };

    ItemFn {
        block: Box::new(transform_block(
            &func.block,
            transform_type,
            gamma,
            &delta,
            block_return_type,
        )),
        ..func.clone()
    }
}
