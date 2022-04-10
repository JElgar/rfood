use syn::*;
use syn::visit_mut::*;
use syn::punctuated::Punctuated;
use syn::token::{Comma, Colon};

use crate::context;
use crate::utils::utils::PopFirst;
use context::gamma::*;
use context::delta::*;
use context::errors::*;

use crate::ast;
use ast::create::*;

use crate::transform;
use transform::visitors::*;

#[derive(clap::ArgEnum, Clone)]
pub enum TransformType {
    OOPToFP,
    FPToOOP,
}

/// Transform a interface (trait) into a datatype (enum)
///
/// This transforms the trait it self as well as the implementations of the trait
pub fn transform_trait(trait_: &ItemTrait, gamma: &mut Gamma) -> Vec<Item> {
    // Create enum varaint for each generator of the trait
    let variants: Vec<syn::Variant> = Vec::from_iter(gamma.get_generators(&trait_.ident).iter().map(|(generator, _)| create_enum_variant(&generator.ident, generator.fields.clone())));

    // Create the enum
    let new_enum = ast::create::create_enum(&trait_.ident, variants, &trait_.generics);
    gamma.add_enum(&new_enum);

    // For each destructor of the trait create a new consumer of the enum 
    let mut consumers = Vec::from_iter(gamma.get_destructors(&trait_.ident).iter().map(|destructor| {
        transform_destructor(trait_, destructor, &new_enum, gamma)
    }));

    let mut items = vec![Item::Enum(new_enum.clone())];
    items.append(&mut consumers);
    return items;
}

pub fn transform_consumer_fn_to_trait_item(consumer: &ItemFn) -> TraitItemMethod {
    TraitItemMethod{
        attrs: consumer.attrs.clone(),
        sig: transform_consumer_signature(&consumer.sig),
        default: None,
        semi_token: Some(token::Semi::default()),
    }
}

pub fn transform_enum(enum_: &ItemEnum, gamma: &mut Gamma) -> Vec<Item> {

    // Create a trait
    let consumers = gamma.get_enum_consumers(enum_);
    let trait_methods: Vec<TraitItemMethod> = Vec::from_iter(consumers.iter().map(|consumer| {
        transform_consumer_fn_to_trait_item(&consumer)
    }));


    let mut trait_ = create_trait(
        &enum_.ident,
        &trait_methods.iter().zip(consumers.iter()).map(|(method, consumer)| {
            TraitItem::Method(TraitItemMethod{
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
                        Some(*consumer.block.clone())
                    }
                },
                ..method.clone()
            })
        }).collect::<Vec<TraitItem>>()
    );
    gamma.add_trait(&trait_);

    let mut items = vec![Item::Trait(trait_.clone())];
        
    // For each variant of the enum create a struct and an impl
    for variant in enum_.variants.iter() {
        // Create the struct
        let struct_ = create_struct(&variant.ident, &enum_.ident, variant.fields.clone());
        gamma.add_struct(&struct_);
        items.push(Item::Struct(struct_.clone()));

        // Add emtpy generator to gamma
        let impl_ = create_impl(&enum_.ident, &variant.ident, Vec::new());
        gamma.add_generator(&trait_, &struct_, &impl_);

        // Collect methods
        // TODO handle trait method
        let impl_items = consumers.iter().zip(trait_methods.iter()).filter_map(|(consumer, trait_method)| {
            // Get the expr for the new destructor
            let consumer_expr: Option<Expr> = match get_match_expr_for_enum(consumer, &variant.ident) {
                // If there is an arm in the match statement, we can use it
                Ok(expr) => Some(expr),
                // Otherwise we will have to use the method body for all the cases
                Err(e) => {
                    // 1. The trait has a default impl for this method. This is only possible if the
                    //    return type of the consumer is not the same as the enum
                    //    In this case we can just use the default impl so no impl is needed here
                    let return_type = consumer.sig.output.get_delta_type(None);
                    if return_type.is_some() && return_type.unwrap().name != enum_.ident {
                        return None;
                    }
                    
                    // 2. Same as above but the return type is the same as the enum. In this case we
                    //    need to copy the default impl each time.
                    //    TODO create a function and use it here instead of duplicate code 
                    Some(Expr::Block(ExprBlock{block: *consumer.block.clone(), attrs: Vec::new(), label: None}))
                }
            };

            // If we dont get an expr for this destructor then we can skip it. (This will be
            // because the expr is already defined in the default impl of the trait)
            if consumer_expr.is_none() {
                return None;
            }
           
            // TODO Cannot collect dleta for this sig here as the generators has not been added yet
            let mut delta = Delta::new();
            delta.collect_for_sig(&consumer.sig, None);


            let expr = transform_consumer_expr(
                &consumer_expr.unwrap(),
                get_fn_arg_name(&consumer.sig.inputs.first().unwrap()),
                Vec::from_iter(variant.fields.iter().map(|field| {
                    field.ident.clone().unwrap()
                })),
                &delta,
                gamma
            );

            Some(
                ImplItem::Method(create_impl_method(&trait_method.sig, &Block{
                    brace_token: token::Brace::default(),
                    stmts: vec![Stmt::Expr(expr)],
                }))
            )
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
fn transform_destructor(trait_: &ItemTrait, destructor: &TraitItemMethod, enum_: &ItemEnum, gamma: &mut Gamma) -> Item {

    // Collect all the generics from all the implementations of the trait destructor
    let mut generics = trait_.generics.clone();
    let enum_generics = trait_.generics.clone();
    for (_, generator_impl) in gamma.get_generators(&trait_.ident) {
        // For the implementation find the generics for the trait
        let trait_generics = get_generics_from_path_segment(&*generator_impl.trait_.unwrap().1.segments.first().unwrap());

        // For the implementation find all the generics 
        let impl_generics = generator_impl.generics;

        // Remove the generics that are from the trait from all the generics of the implementation
        let struct_generics: Vec<GenericParam> = Vec::from_iter(impl_generics.params.iter().filter(|param| !trait_generics.params.iter().any(|trait_param| trait_param == *param)).cloned());

        // Add the generics that a just for the struct (not from the trait) to the list of generics
        generics.params.extend(struct_generics);
    }

    let (mut signature, enum_instance_name) = transform_destructor_signature(&destructor.sig, &enum_.ident, &generics, &enum_generics, &gamma);

    let mut arms: Vec<syn::Arm> = Vec::new();
    // If any of the impl do not have an implementation of the destructor then we need to create a
    // wildcard argument
    let mut wild_card_arm_required = false;
    for (generator, generator_impl) in gamma.get_generators(&trait_.ident).iter() {

        // TODO Create delta for destructor method
        let mut old_delta = Delta::new();
        old_delta.collect_for_sig(&destructor.sig, Some(&generator.ident));

        let mut new_delta = Delta::new();
        new_delta.collect_new_for_destructor_impl(&signature, &generator);

        let result = transform_destructor_impl(generator, destructor, &enum_.ident, &enum_instance_name, generator_impl, gamma, &old_delta, &new_delta, &signature);

        match result {
            Ok(arm) => arms.push(arm),
            Err(NotFound { .. }) => wild_card_arm_required = true,
        }
    };

    // If required, add the wild card arm
    if wild_card_arm_required {
        let mut old_delta = Delta::new();
        old_delta.collect_for_sig(&destructor.sig, Some(&trait_.ident));
        
        let mut new_delta = Delta::new();
        new_delta.collect_for_sig(&signature, None);

        // Get impl in the trait
        let mut body = Expr::Block(ExprBlock{block: Gamma::get_destructor_impl_for_trait(trait_, &destructor.sig.ident).unwrap().default.unwrap(), attrs: Vec::new(), label: None});
    
        body = transform_destructor_expr(&body, &old_delta, &new_delta, gamma, &trait_.ident, EType::DeltaType(signature.output.get_delta_type(None).unwrap()));
        
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
    
    let func = ast::create::create_function(signature, vec![Stmt::Expr(match_expr)]);
    gamma.add_enum_consumer(&enum_, &destructor.sig.ident, &func);
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
fn transform_destructor_impl(generator: &ItemStruct, destructor: &TraitItemMethod, enum_name: &Ident, enum_instance_name: &Ident, impl_: &ItemImpl, gamma: &Gamma, old_delta: &Delta, new_delta: &Delta, consumer_signature: &Signature) -> std::result::Result<Arm, NotFound> {
    // Find the implementation of the method
    let method_result = Gamma::get_destructor_impl_for_generator(&impl_, &destructor.sig.ident);
    if method_result.is_err() {
        return Err(method_result.err().unwrap());
    }

    let mut expr: Expr  = Expr::Block(ExprBlock{block: method_result.unwrap().block, attrs: Vec::new(), label: None});

    // Transform the body of the method
    expr = transform_destructor_expr(&expr, &old_delta, &new_delta, gamma, enum_instance_name, EType::DeltaType(consumer_signature.output.get_delta_type(None).unwrap()));

    // Create the arm of the match statement
    let path = ast::create::create_path_for_enum(enum_name, &generator.ident);
    Ok(ast::create::create_match_arm(
        path, get_struct_attrs(&generator), expr,
    ))
}

/// Given expression for destructor covert all method calls 
///
/// Replace all method calls to the destructor with the corresponding consumer function call
fn transform_destructor_expr(expr: &Expr, old_delta: &Delta, new_delta: &Delta, gamma: &Gamma, enum_name: &Ident, output_type: EType) -> Expr {
    let mut expr_clone = expr.clone();
    
    let mut rfc = ReplaceFieldCalls{delta: old_delta.clone()};
    rfc.visit_expr_mut(&mut expr_clone);

    let mut rmc = ReplaceMethodCalls{delta: old_delta.clone(), gamma: gamma.clone()};
    rmc.visit_expr_mut(&mut expr_clone);

    let mut rs = ReplaceSelf{enum_name: enum_name.clone()};
    rs.visit_expr_mut(&mut expr_clone);

    expr_clone = transform_expr(&mut expr_clone, &TransformType::OOPToFP, gamma, &new_delta, output_type);
    return expr_clone; 
}

fn transform_consumer_expr(expr: &Expr, self_arg_name: Ident, trait_attributes: Vec<Ident>, delta: &Delta, gamma: &Gamma) -> Expr {
    let mut expr_clone = expr.clone();
    let mut tc = TransformConsumer{trait_attributes, gamma: gamma.clone(), self_arg_name};
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
/// Returns the function signature and the name of the type which replaces self if self is present
fn transform_destructor_signature(signature: &Signature, enum_name: &Ident, generics: &Generics, enum_generics: &Generics, gamma: &Gamma) -> (Signature, Ident){
    let enum_instance_name = transform_type_to_name(enum_name);

    // Transform arguments
    let new_inputs = syn::punctuated::Punctuated::from_iter(signature.inputs.iter().map(|item| {
        let create_self_consumer_signature = |as_ref| {
            // Add generics to enum_name
            create_consumer_signature(enum_name, &enum_instance_name, as_ref, &enum_generics)
        };

        // Replace self with enum
        if let syn::FnArg::Receiver(..) = item {
            // TODO use borrow as required and check this is acutally self
            return create_self_consumer_signature(true);
        }
        if let syn::FnArg::Typed(PatType{
            pat,
            ty,
            ..
        }) = item {
            if let Pat::Ident(pat_ident) = &**pat {
              // The type of the thing
              let arg_type = ty.get_delta_type();

              // Check if the type is in the geneators
              if gamma.is_interface(&arg_type.name) {
                  // If self
                  if arg_type.name == "Self" || pat_ident.ident == "self" {
                      return create_self_consumer_signature(false);
                  }
                  return create_consumer_signature(&arg_type.name, &pat_ident.ident, false, &enum_generics);
              }
            }
        }
        item.clone()
    }));

    // NOTE output transfomration is handled later
    
    (
        syn::Signature {
            inputs: new_inputs,
            generics: generics.clone(),
            ..signature.clone()
        },
        enum_instance_name.clone()
    )
}

pub fn transform_consumer_signature(signature: &Signature) -> Signature {
    let mut inputs = signature.inputs.clone();

    // Get the self arg
    let consumer_arg: FnArg = inputs.iter().next().unwrap().clone();
    let self_type = consumer_arg.get_delta_type(None);

    // Ignoring the first element transfrom each argument
    let mut new_inputs = Vec::from_iter(inputs.iter().skip(1).map(|arg| {
        // TODO make all args with type of enum, Box<dyn T>
        if arg.get_delta_type(None).name == self_type.name && self_type.ref_type != RefType::Box {
            // Create box dyn of fn arg
            return create_dyn_box_arg(&arg);
        }
        arg.clone()
    }));
    new_inputs.insert(0, create_self_fn_arg(if consumer_arg.get_ref_type() == RefType::Ref {RefType::Ref} else {RefType::Box}));

    let mut output = signature.output.clone();

    if let ReturnType::Type(ra, ty) = &output {
        if ty.get_delta_type().name == self_type.name {
            // Create box dyn of fn arg
            output = ReturnType::Type(*ra, Box::new(create_dyn_box_of_type(&ty)))
        }
    }

    Signature {
        inputs: syn::punctuated::Punctuated::from_iter(new_inputs),
        output,
        ..signature.clone()
    }
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

fn transform_struct_instantiation_path_for_enum(expr_struct: &ExprStruct, gamma: &Gamma, delta: &Delta) -> Path {
    // Get the name of the enum
    let trait_name = gamma.get_generator_trait(&expr_struct.path.get_delta_type().name).unwrap();
    // Add the enum in front of the struct
    let mut new_path_vec = vec![PathSegment{ident: trait_name.ident.clone(), arguments: PathArguments::None}];
    new_path_vec.append(&mut Vec::from_iter(expr_struct.path.segments.clone().iter().cloned()));

    // Add it to the front of the struct path segments
    Path {
        segments: Punctuated::from_iter(new_path_vec),
        ..expr_struct.path.clone()
    }
}

fn transform_expr_type(expr: &Expr, current_type: &DeltaType, required_type: &DeltaType, gamma: &Gamma) -> Expr {
    if current_type.is_equaivalent(&required_type, &gamma) {
        expr.clone()
    } else if current_type.ref_type == RefType::Box && required_type.ref_type == RefType::None {
        create_dereference_of_expr(expr)
    } else if current_type.ref_type == RefType::None && required_type.ref_type == RefType::Box {
        create_box_of_expr(expr)
    } else {
        panic!("Cannot transform {:?} to {:?}", current_type, required_type)
    }
}

fn transform_method_call_arguments(method_call: &ExprMethodCall, gamma: &Gamma, delta: &Delta) -> Punctuated<Expr, Comma> {
    let reciever_ident = delta.get_type_of_expr(&method_call.receiver, gamma).unwrap().name;
    let method_ident = method_call.method.clone();

    // Get the old and new signature
    let old_signature = gamma.get_destructor_signature(&reciever_ident, &method_ident);
    let transformed_signature = gamma.get_transformed_destructor_signature(&reciever_ident, &method_ident);

    // Iterate over all the arguments of the method, ignoring the first argment of both the
    // signatures (as the self arg is handled separately)
    Punctuated::from_iter(method_call.args.iter().zip(old_signature.inputs.iter().skip(1)).zip(transformed_signature.inputs.iter().skip(1)).map(|((arg, old_fn_arg), new_fn_arg)| {
        // If the old arg is a box type and the new arg is not
        if old_fn_arg.get_ref_type() == RefType::Box && new_fn_arg.get_ref_type() == RefType::None {
            create_dereference_of_expr(&arg.clone())
        } else {
            arg.clone()
        }
    }))
}

fn transform_block(block: &Block, transform_type: &TransformType, gamma: &Gamma, delta: &Delta, return_type: EType) -> Block {
    let mut delta = delta.clone();
    let stmts = Vec::from_iter(block.stmts.iter().enumerate().map(|(index, stmt)| transform_statement(&stmt, transform_type, gamma, &mut delta, if index == block.stmts.len() - 1 {return_type.clone()} else {EType::None})));
    Block{
        stmts,
        ..block.clone()
    }
}

fn transform_expr(expr: &Expr, transform_type: &TransformType, gamma: &Gamma, delta: &Delta, return_type: EType) -> Expr {
    // Clone the delta at this stage
    let mut delta = delta.clone();
    match (transform_type, expr) {
        (_, Expr::Unary(_)) => {
            // Remove any existing derefs so we can fix the type manually
            let expr = clean_type(expr);
            let current_type = delta.get_type_of_expr(&expr, gamma);
            if let EType::DeltaType(delta_type) = return_type {
                if let Ok(current_type) = current_type {
                    return transform_expr_type(&expr, &current_type, &delta_type, gamma);
                }
            }
            return expr.clone();
        }
        (TransformType::OOPToFP, Expr::MethodCall(expr_method_call))
            if gamma.is_generator_type(&delta.get_type_of_expr(&expr_method_call.receiver, gamma).unwrap().name) 
        => {
            let ExprMethodCall { receiver, method, .. } = expr_method_call;
            let receiver_expr = if delta.get_type_of_expr(&receiver, gamma).unwrap().ref_type == RefType::Box {
                create_dereference_of_expr(&receiver)
            } else {
                *receiver.clone()
            };

            let mut new_args = Punctuated::from_iter(vec![receiver_expr]);
            new_args.extend(transform_method_call_arguments(&expr_method_call, gamma, &delta));
            create_function_call(&method, new_args)
        },
        // Any other method call, transform all the args and the receiver
        (_, Expr::MethodCall(method_call)) => {
            // Get the signature of the method call, NOTE this will fail if any method call are
            // made which are not on destructors TODO fix
            let reciever_type = delta.get_type_of_expr(&method_call.receiver, &gamma).unwrap();
            let signature = gamma.get_destructor_signature(&reciever_type.name, &method_call.method);
            Expr::MethodCall(ExprMethodCall{
                receiver: Box::new(transform_expr(&method_call.receiver, transform_type, gamma, &delta, EType::Any)),
                args: Punctuated::from_iter(method_call.args.iter().enumerate().map(|(index, arg)| 
                    transform_expr(
                        arg,
                        transform_type,
                        gamma,
                        &delta,
                        EType::DeltaType(signature.inputs[index].get_delta_type(Some(reciever_type.name.clone()))),
                    )
                )),
                ..method_call.clone()
            })
        },
        // If the experssion is calling a consumer and we are transforming from FP to OOP
        // Then we should replace the call with a method call
        (TransformType::FPToOOP, Expr::Call(expr_call)) if gamma.is_consumer(&get_function_call_name(expr_call)) => {
            // Extract the first argument to the function
            let mut args = expr_call.args.clone();
            let first_arg = clean_type(&args.pop_first().unwrap());


            // Create method call
            let method_call = create_method_call(&get_function_call_name(expr_call), &first_arg, &args);
            // Performance regular transformations to the new method call (fix typing of args)
            transform_expr(&method_call, &transform_type, &gamma, &delta, return_type)
        },
        (_, Expr::Call(expr_call)) => {
            // If the expression is a box call and we dont want a box, remove the box
            if let Ok(inner_expr) = new_box_call_expr(expr) {
                println!("Transforming box call expr");
                // And the expected return type is not a box or the inner type is a box, then
                if let EType::DeltaType(dt) = &return_type {
                    println!("Really transforming box call expr {:?}", dt.ref_type);
                    println!("Inner type {:?}, expr: {:?}", delta.get_type_of_expr(&inner_expr, gamma).unwrap().ref_type, inner_expr);
                    if dt.ref_type != RefType::Box || delta.get_type_of_expr(&inner_expr, gamma).unwrap().ref_type == RefType::Box {
                        println!("Actually transforming box call expr");
                        // Then we should transform the expression to a method call
                        return transform_expr(&inner_expr, &transform_type, &gamma, &delta, return_type);
                    }
                }
            }

            if let ExprCall { func: box Expr::Path (ExprPath{path, ..}), ..} = expr_call {
                let signature = gamma.get_signature(&get_function_call_name(expr_call));

                return Expr::Call(ExprCall{
                    func: Box::new(transform_expr(&expr_call.func, transform_type, gamma, &delta, EType::Any)),
                    args: Punctuated::from_iter(expr_call.args.iter().enumerate().map(|(index, arg)| 
                        transform_expr(
                            arg,
                            transform_type,
                            gamma,
                            &delta,
                            match &signature {
                                Ok(sig) => EType::DeltaType(sig.inputs[index].get_delta_type(None)),
                                Err(_) =>  EType::Any
                            }
                        )
                    )),
                    ..expr_call.clone()
                });
            }
            panic!("Cannot transform non path calls")
        },
        (_, Expr::Return(expr_return)) if expr_return.expr.is_some() => {
            Expr::Return(ExprReturn{
                expr: Some(Box::new(transform_expr(&expr_return.clone().expr.unwrap(), transform_type, gamma, &delta, return_type))),
                ..expr_return.clone()
            })
        }
        (OOPToFP, Expr::Struct(expr_struct)) if gamma.get_generator_trait(&expr_struct.path.get_delta_type().name).is_some() => {
            let struct_ = Expr::Struct(ExprStruct{
                path: transform_struct_instantiation_path_for_enum(expr_struct, gamma, &delta),
                fields: Punctuated::from_iter(expr_struct.fields.iter().map(|field| {

                    // Get the enum 
                    let enum_variant_ident = expr_struct.path.get_delta_type().name;
                    let enum_variant = gamma.get_enum_variant(&enum_variant_ident, &enum_variant_ident);
                    let mut enum_delta = Delta::new();
                    enum_delta.collect_for_enum_variant(&enum_variant);

                    let required_type = enum_delta.get_type_of_member(&field.member);
                    let new_expr = transform_expr(&field.expr, transform_type, gamma, &delta, EType::DeltaType(required_type.clone()));

                    // Check type of expr matches required type
                    let new_expr_type = delta.get_type_of_expr(&new_expr, gamma).unwrap();

                    FieldValue{
                        expr: transform_expr_type(&new_expr, &new_expr_type, &required_type, &gamma),
                        ..field.clone()
                    }
                })),
                ..expr_struct.clone()
            });
            if let EType::DeltaType(dt) = return_type {
                return transform_expr_type(&struct_, &delta.get_type_of_expr(&struct_, gamma).unwrap(), &dt, &gamma);
            }
            return struct_;
        },
        (FPToOOP, Expr::Struct(expr_struct)) if gamma.is_enum_or_variant(&expr_struct.path.get_delta_type().name) => {
            let struct_ = Expr::Struct(ExprStruct{
                path: Path{
                    leading_colon: None,
                    segments: Punctuated::from_iter(vec![expr_struct.path.segments.last().unwrap().clone()]),
                },
                // TODO update the transformation so that it does the things (this does the wrong
                // way things)
                // TODO DO DOODODOD 
                // THEN add in renames to transform expr and try and use it to rename the self
                // stuff (where set is renamed to self)
                fields: Punctuated::from_iter(expr_struct.fields.iter().map(|field| {

                    // Get the enum 
                    let struct_ident = get_path_call_name(&expr_struct.path);
                    let struct_ = gamma.get_struct_by_name(&struct_ident);
                    let mut struct_delta = Delta::new();
                    struct_delta.collect_for_struct(&struct_);

                    let required_type = struct_delta.get_type_of_member(&field.member);
                    let new_expr = transform_expr(&field.expr, transform_type, gamma, &delta, EType::DeltaType(required_type.clone()));

                    // Check type of expr matches required type
                    let new_expr_type = delta.get_type_of_expr(&new_expr, gamma).unwrap();

                    FieldValue{
                        // TODO Move this transform type into transform expr
                        expr: transform_expr_type(&new_expr, &new_expr_type, &required_type, &gamma),
                        ..field.clone()
                    }
                })),
                ..expr_struct.clone()
            });
            if let EType::DeltaType(dt) = return_type {
                return transform_expr_type(&struct_, &delta.get_type_of_expr(&struct_, gamma).unwrap(), &dt, &gamma);
            }
            return struct_;
        },
        (_, Expr::Block(expr_block)) => {
            Expr::Block(ExprBlock{
                block: transform_block(&expr_block.block, transform_type, gamma, &delta, return_type),
                ..expr_block.clone()
            })
        },
        _ => {
            // println!("Skipping unsupported {:?} with delta {:?}", expr, delta);
            expr.clone()
        }
    }
}
 
fn transform_statement(statement: &Stmt, transform_type: &TransformType, gamma: &Gamma, delta: &mut Delta, return_type: EType) -> Stmt {
    match statement {
        Stmt::Local(local) => {
            delta.collect_for_local(&local, gamma);
            let init_unwrap = local.init.as_ref().unwrap();
            Stmt::Local(Local{
                init: Some((
                    init_unwrap.0,
                    Box::new(transform_expr(&init_unwrap.1, transform_type, &gamma, delta, EType::Any))
                )),
                ..local.clone()
            })
        },
        Stmt::Semi(expr, semi) => {
            Stmt::Semi(transform_expr(&expr, transform_type, gamma, delta, return_type), *semi)
        },
        Stmt::Expr(expr) => {
            Stmt::Expr(transform_expr(&expr, transform_type, gamma, delta, return_type))
        },
        _ => panic!("Unsupported statement {:?}", statement)
    }
}

pub fn transform_item(item: &syn::Item, transform_type: &TransformType, gamma: &Gamma) -> syn::Item {
    match item {
        Item::Fn(item_fn) => Item::Fn(transform_function(item_fn, transform_type, gamma)),
        Item::Impl(item_impl) => {
            let for_type = item_impl.self_ty.get_delta_type().name;
            Item::Impl(ItemImpl{
                items: item_impl.items.iter().map(|item| {
                    transform_impl_item(item, &for_type, transform_type, gamma)
                }).collect(),
                ..item_impl.clone()
            })
        },
        _ => {
            item.clone()
        },
    }
}

pub fn transform_impl_item(impl_item: &syn::ImplItem, impl_for_type: &Ident, transform_type: &TransformType, gamma: &Gamma) -> syn::ImplItem {
    match impl_item {
        ImplItem::Method(impl_item_method) => {
            let return_type = impl_item_method.sig.output.get_delta_type(Some(impl_for_type.clone()));
            let block_return_type = match return_type {
                Some(rt) => EType::DeltaType(rt),
                None => EType::None
            };

            ImplItem::Method(
                ImplItemMethod {
                    block: {
                        let mut delta = Delta::new();
                        println!("\nCollecting");
                        delta.collect_for_sig(&impl_item_method.sig, Some(impl_for_type));
                        println!("");
                        println!("{:?}", delta);
                        println!("");

                        transform_block(&impl_item_method.block, transform_type, gamma, &delta, block_return_type)
                    },
                    ..impl_item_method.clone()
                }
            )
        },
        _ => {
            // println!("Skipping unsupported {:?}", impl_item);
            impl_item.clone()
        },
    }
}
 
/// Transform all the statements in a fuction
fn transform_function(func: &ItemFn, transform_type: &TransformType, gamma: &Gamma) -> syn::ItemFn {
    let mut delta = Delta::new();
    delta.collect_for_sig(&func.sig, None);

    let return_type = func.sig.output.get_delta_type(None);
    let block_return_type = match return_type {
        Some(rt) => EType::DeltaType(rt),
        None => EType::None
    };

    ItemFn { 
        block: Box::new(transform_block(&func.block, transform_type, gamma, &delta, block_return_type)),
        ..func.clone()
    }
}
