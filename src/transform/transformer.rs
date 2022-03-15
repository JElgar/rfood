use syn::*;
use syn::visit_mut::*;

use crate::context;
use context::gamma::Gamma;
use context::delta::{Delta, get_struct_attrs, get_type_ident_from_type};

use crate::ast;
use ast::create::{create_enum_variant, create_consumer_signature};

use crate::transform;
use transform::visitors::*;

pub fn transform_trait(trait_: &ItemTrait, gamma: &Gamma) -> Vec<Item> {
    // Create enum varaint for each generator of the trait
    let variants: Vec<syn::Variant> = Vec::from_iter(gamma.get_generators(trait_).iter().map(|(generator, _)| create_enum_variant(&generator.ident, generator.fields.clone())));

    // Create the enum
    let new_enum: syn::Item = ast::create::create_enum(&trait_.ident, variants);
    // For each destructor of the trait create a new consumer of the enum 
    let mut consumers = Vec::from_iter(gamma.get_destructors(trait_).iter().map(|destructor| {
        transform_destructor(trait_, destructor, &trait_.ident, gamma)
    }));

    let mut items = vec![new_enum];
    items.append(&mut consumers);
    return items;
}


fn transform_destructor(trait_: &ItemTrait, destructor: &TraitItemMethod, enum_name: &Ident, gamma: &Gamma) -> Item {
    let (signature, enum_instance_name) = transform_destructor_signature(&destructor.sig, enum_name, &gamma);
    let arms: Vec<syn::Arm> = Vec::from_iter(gamma.get_generators(trait_).iter().map(|(generator, generator_impl)| {
        transform_destructor_impl(generator, destructor, enum_name, generator_impl)
    }));

    let match_expr = ast::create::create_match_statement(&enum_instance_name, arms);
    ast::create::create_function(signature, vec![Stmt::Expr(match_expr)])
}

fn transform_destructor_impl(generator: &ItemStruct, destructor: &TraitItemMethod, enum_name: &Ident, impl_: &ItemImpl) -> Arm {
    let method: ImplItemMethod = Gamma::get_destructor_impl_for_generator(&impl_, destructor);
    let mut delta: Delta = Delta::new();

    delta.collect_for_destructor_impl(&method, generator);

    // Extract the body of the method
    let mut expr: Expr  = Expr::Block(ExprBlock{block: method.block, attrs: Vec::new(), label: None});
    expr = transform_destructor_expr(&expr, &delta);

    let path = ast::create::create_match_path_for_enum(enum_name, &generator.ident);
    ast::create::create_match_arm(
        path, get_struct_attrs(&generator), expr,
    )
}

/// Given expression for destructor covert all method calls 
///
/// Replace all method calls to the destructor with the corresponding consumer function call
fn transform_destructor_expr(expr: &Expr, delta: &Delta) -> Expr {

    let mut expr_clone = expr.clone();
    let mut rfc = ReplaceFieldCalls{delta: delta.clone()};
    rfc.visit_expr_mut(&mut expr_clone);

    let mut rmc = ReplaceMethodCalls{delta: delta.clone()};
    rmc.visit_expr_mut(&mut expr_clone);
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
fn transform_destructor_signature(signature: &Signature, enum_name: &Ident, gamma: &Gamma) -> (Signature, Ident){
    let enum_instance_name = transform_type_to_name(enum_name);
    let new_inputs = syn::punctuated::Punctuated::from_iter(signature.inputs.iter().map(|item| {
        if let syn::FnArg::Receiver(arg_data) = item {
            return create_consumer_signature(enum_name, &enum_instance_name);
        }
        if let syn::FnArg::Typed(PatType{
            pat,
            ty,
            ..
        }) = item {
            if let Pat::Ident(pat_ident) = &**pat {
              println!("Got ident type {:?}", pat_ident.ident);
              // The type of the thing
              let arg_type = get_type_ident_from_type(&*ty);
              println!("Got arg_type type {:?}", arg_type);

              // Check if the type is in the geneators
              if gamma.is_interface(&arg_type) {
                  // If yes transform
                  return create_consumer_signature(enum_name, &enum_instance_name);
              }
            }
        }
        println!("Skipped transforming signature with inputs: {:?}", signature.inputs);
        item.clone()
    }));
    
    (
        syn::Signature {
            inputs: new_inputs,
            ..signature.clone()
        },
        enum_instance_name.clone()
    )
}

fn transform_type_to_name(type_ident: &Ident) -> Ident {
    Ident::new(&type_ident.to_string().to_lowercase(), type_ident.span())
}
