#![feature(rustc_private)]

extern crate syn;
extern crate proc_macro;
#[macro_use]
extern crate quote;

extern crate rustc_ast;
extern crate rustc_typeck;

// use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

mod ast;
mod context;
mod fp;
mod oop;
mod transform;

use ast::print::write_and_fmt;
use context::gamma::{Gamma, generate_gamma};
use transform::transformer::transform_trait;

fn print_goal() {
  // -- Print current and goal enum --//
  let filename = "./src/test.rs";
  let mut file = File::open(&filename).expect("Unable to open file");

  let mut src = String::new();
  file.read_to_string(&mut src).expect("Unable to read file");

  let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
  println!("{:?}", syntax);
}

fn visitor_test() {
  let filename = "./src/test.rs";
  let mut file = File::open(&filename).expect("Unable to open file");

  let mut src = String::new();
  file.read_to_string(&mut src).expect("Unable to read file");
}

fn main() {
    // context::delta::Delta::generate_for_file();
    // return;

    // print_goal();
    // println!();
    // println!();

    // Environemnt map
    
    //-- Do the transfrom --//
    let filename = "./src/oop/set.rs";
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    let mut syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    
    let gamma: Gamma = generate_gamma(&syntax);
    
    for trait_ in &gamma.traits {
        syntax.items.append(&mut transform_trait(&trait_, &gamma));
    }

    if write_and_fmt("outputs/output.rs", quote!(#syntax)).is_err() {
        panic!("Unable to write output file");
    }

    // // Create enum from trait and its impls
    // for trait_ in &gamma.traits {
    // //for trait_ in &traits {
    //     let mut variants: Vec<syn::Variant> = gamma.get_trait_variants(trait_);

    //     for variant in &gamma.get_trait_variants(trait_) {
    //         let impl_struct = get_struct(&syntax, &variant.name);
    //         variants.push(ast::create::create_enum_variant(&variant.name, impl_struct.fields));
    //     }

    //     // For each method in the trait find the matching implementation in each impl
    //     for method in &trait_.methods {
    //         let (fp_signature, self_indent) = trait_signature_to_fp_function_signature(method.sig.clone(), &trait_.name);

    //         let mut function_stmts: Vec<syn::Stmt> = Vec::new();

    //         // Create a match statement which matches on the enum and uses the method
    //         match self_indent {
    //             Some(ident) => {
    //                 let mut arms: Vec<syn::Arm> = Vec::new();
    //                 // For each impl create a match arm
    //                 for impl_ in &trait_.impls {
    //                     let expr: syn::Expr = get_impl_method_expression(get_matching_impl_method(method, impl_));
    //                     let path = ast::create::create_match_path_for_enum(&trait_.name, &impl_.name);
    //                     let match_arm = ast::create::create_match_arm(
    //                         path, Vec::new(), expr,
    //                     );
    //                     arms.push(match_arm);
    //                 }


    //                 let match_expr = ast::create::create_match_statement(ident, arms);
    //                 function_stmts.push(
    //                     syn::Stmt::Expr(match_expr)
    //                 );
    //             },
    //             None => {panic!("Cannot yet transform method with no self")}
    //         }
    //         
    //         // Create a method with the functional programming function signature
    //         let function = ast::create::create_function(fp_signature, function_stmts);
    //         syntax.items.push(function);
    //     }

    //     let new_enum: syn::Item = ast::create::create_enum(&trait_.name, variants);
    //     syntax.items.push(new_enum);
    // }

    // // TODO: https://stackoverflow.com/questions/65764987/how-to-pretty-print-syn-ast
    // println!("{}", quote!(#syntax));

    // ReplaceSelfMethodCall.visit_file_mut(&mut syntax);
    // ReplaceSelf.visit_file_mut(&mut syntax);

    // if write_and_fmt("outputs/output.rs", quote!(#syntax)).is_err() {
    //     panic!("Unable to write output file");
    // }
}
