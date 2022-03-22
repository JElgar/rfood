use syn::*;
use syn::visit_mut::*;
use syn::punctuated::Punctuated;
use syn::token::{Comma, Colon};

use crate::context;
use context::gamma::{Gamma, get_generics_from_path_segment};
use context::delta::*;

use crate::ast;
use ast::create::*;

use crate::transform;
use transform::visitors::*;

/// Transform a interface (trait) into a datatype (enum)
///
/// This transforms the trait it self as well as the implementations of the trait
pub fn transform_trait(trait_: &ItemTrait, gamma: &Gamma) -> Vec<Item> {
    // Create enum varaint for each generator of the trait
    let variants: Vec<syn::Variant> = Vec::from_iter(gamma.get_generators(trait_).iter().map(|(generator, _)| create_enum_variant(&generator.ident, generator.fields.clone())));

    // Create the enum
    let new_enum: syn::Item = ast::create::create_enum(&trait_.ident, variants, &trait_.generics);

    // For each destructor of the trait create a new consumer of the enum 
    let mut consumers = Vec::from_iter(gamma.get_destructors(trait_).iter().map(|destructor| {
        transform_destructor(trait_, destructor, &trait_.ident, gamma)
    }));

    let mut items = vec![new_enum];
    items.append(&mut consumers);
    return items;
}

/// Transforms a destructor of a trait into a consumer of the enum
///
/// * `trait_` - The trait that the destructor belongs to
/// * `destructor` - The destructor to transform
/// * `enum_ident` - The ident of the enum that the new generator should be created for 
/// * `gamma` - The gamma context
fn transform_destructor(trait_: &ItemTrait, destructor: &TraitItemMethod, enum_name: &Ident, gamma: &Gamma) -> Item {

    // Collect all the generics from all the implementations of the trait destructor
    let mut generics = trait_.generics.clone();
    let enum_generics = trait_.generics.clone();
    for (_, generator_impl) in gamma.get_generators(trait_) {
        // For the implementation find the generics for the trait
        let trait_generics = get_generics_from_path_segment(&*generator_impl.trait_.unwrap().1.segments.first().unwrap());

        // For the implementation find all the generics 
        let impl_generics = generator_impl.generics;

        // Remove the generics that are from the trait from all the generics of the implementation
        let struct_generics: Vec<GenericParam> = Vec::from_iter(impl_generics.params.iter().filter(|param| !trait_generics.params.iter().any(|trait_param| trait_param == *param)).cloned());

        // Add the generics that a just for the struct (not from the trait) to the list of generics
        generics.params.extend(struct_generics);
    }

    let (mut signature, enum_instance_name) = transform_destructor_signature(&destructor.sig, enum_name, &generics, &enum_generics, &gamma);

    let arms: Vec<syn::Arm> = Vec::from_iter(gamma.get_generators(trait_).iter().map(|(generator, generator_impl)| {
        transform_destructor_impl(generator, destructor, enum_name, &enum_instance_name, generator_impl)
    }));

    let mut match_expr = ast::create::create_match_statement(&enum_instance_name, arms);
   
    
    if is_dyn_box_generator_return(&signature, gamma) {
        println!("It is a dynbox generator");
        // TODO change return type
        signature = Signature {
            output: transform_dyn_box_destructor_signature_output(&signature.output),
            ..signature
        };

        // TODO Change every return statement
        let mut rdbdrs = ReplaceDynBoxDestructorReturnStatements;
        rdbdrs.visit_expr_mut(&mut match_expr);
    }

    ast::create::create_function(signature, vec![Stmt::Expr(match_expr)])
}

pub fn transform_dyn_box_destructor_signature_output(output: &ReturnType) -> ReturnType {
    if let ReturnType::Type(_, type_) = output {
        return create_return_type_from_ident(&get_type_ident_from_type(type_));
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
fn transform_destructor_impl(generator: &ItemStruct, destructor: &TraitItemMethod, enum_name: &Ident, enum_instance_name: &Ident, impl_: &ItemImpl) -> Arm {
    // Find the implementation of the method
    let method: ImplItemMethod = Gamma::get_destructor_impl_for_generator(&impl_, destructor);

    // Generate delta for the method
    let mut delta: Delta = Delta::new();
    delta.collect_for_destructor_impl(&method, generator);

    // Extract the body of the method
    let mut expr: Expr  = Expr::Block(ExprBlock{block: method.block, attrs: Vec::new(), label: None});

    // Transform the body of the method
    expr = transform_destructor_expr(&expr, &delta, enum_instance_name);

    // Create the arm of the match statement
    let path = ast::create::create_match_path_for_enum(enum_name, &generator.ident);
    ast::create::create_match_arm(
        path, get_struct_attrs(&generator), expr,
    )
}

/// Given expression for destructor covert all method calls 
///
/// Replace all method calls to the destructor with the corresponding consumer function call
fn transform_destructor_expr(expr: &Expr, delta: &Delta, enum_name: &Ident) -> Expr {

    let mut expr_clone = expr.clone();
    let mut rfc = ReplaceFieldCalls{delta: delta.clone()};
    rfc.visit_expr_mut(&mut expr_clone);

    let mut rmc = ReplaceMethodCalls{delta: delta.clone()};
    rmc.visit_expr_mut(&mut expr_clone);

    let mut rs = ReplaceSelf{enum_name: enum_name.clone()};
    rs.visit_expr_mut(&mut expr_clone);

    return expr_clone; 
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
            // TODO use borrow as required
            return create_self_consumer_signature(true);
        }
        if let syn::FnArg::Typed(PatType{
            pat,
            ty,
            ..
        }) = item {
            if let Pat::Ident(pat_ident) = &**pat {
              // The type of the thing
              let arg_type = get_type_ident_from_type(&*ty);

              // Check if the type is in the geneators
              if gamma.is_interface(&arg_type) {
                  // If self
                  if arg_type == "Self" || pat_ident.ident == "self" {
                      return create_self_consumer_signature(false);
                  }
                  return create_consumer_signature(&arg_type, &pat_ident.ident, false, &enum_generics);
              }
            }
        }
        item.clone()
    }));

    // TODO Transform return type 
    // If it returns a box of itself, replace it with the enum
    
    (
        syn::Signature {
            inputs: new_inputs,
            generics: generics.clone(),
            ..signature.clone()
        },
        enum_instance_name.clone()
    )
}

/// Given the name of a type get a sensible name for the object
///
/// * `type_ident` - The name of the type e.g. Foo
///
/// # Examples
///
/// ```
/// let type_ = Ident::new("Something", Span::call_site());
/// assert_eq!(transform_type_to_name(type_).to_string(), "something");
/// ```
fn transform_type_to_name(type_ident: &Ident) -> Ident {
    Ident::new(&type_ident.to_string().to_lowercase(), type_ident.span())
}
