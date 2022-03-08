use syn::*;

use crate::context;
use context::gamma::Gamma;

use crate::ast;
use ast::create::{create_enum_variant, create_consumer_signature};

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
    let (signature, enum_instance_name) = transform_destructor_signature(&destructor.sig, enum_name);
    let arms: Vec<syn::Arm> = Vec::from_iter(gamma.get_generators(trait_).iter().map(|(_, generator_impl)| {
        transform_destructor_impl(trait_, destructor, enum_name, generator_impl)
    }));

    let match_expr = ast::create::create_match_statement(&enum_instance_name, arms);
    ast::create::create_function(signature, vec![Stmt::Expr(match_expr)])
}

fn transform_destructor_impl(trait_: &ItemTrait, destructor: &TraitItemMethod, enum_name: &Ident, impl_: &ItemImpl) -> Arm {
    let expr: Expr = Gamma::get_destructor_impl_for_generator(&impl_, destructor);
    let path = ast::create::create_match_path_for_enum(&trait_.ident, enum_name);
    ast::create::create_match_arm(
        path, Vec::new(), expr,
    )
}

/// Convert signature of destructor to consumer signature
///
/// Replace &self with Box<T> and replace self with T
///
/// * `signature` - The signature of the trait method
/// * `enum_name` - The name of the enum (interface) which replaces self
///
/// Returns the function signature and the name of the type which replaces self if self is present
fn transform_destructor_signature(signature: &Signature, enum_name: &Ident) -> (Signature, Ident){
    let enum_instance_name = transform_type_to_name(enum_name);
    let new_inputs = syn::punctuated::Punctuated::from_iter(signature.inputs.iter().map(|item| {
      if let syn::FnArg::Receiver(arg_data) = item {
          return create_consumer_signature(arg_data, enum_name, &enum_instance_name);
      }
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
