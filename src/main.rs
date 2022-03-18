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

mod ast;
mod context;
mod transform;
mod examples;

use ast::print::write_and_fmt;
use context::gamma::{Gamma, generate_gamma};
use transform::transformer::transform_trait;

#[allow(dead_code)]
fn print_goal() {
  // -- Print current and goal enum --//
  // let filename = "./src/examples/generics/oop.rs";
  let filename = "./src/test.rs";
  let mut file = File::open(&filename).expect("Unable to open file");

  let mut src = String::new();
  file.read_to_string(&mut src).expect("Unable to read file");

  let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
  println!("{:?}\n\n", syntax);
}

fn transform(filename: &str) {
    //-- Do the transfrom --//
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    let mut syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
   
    // Generate global gamma context
    let gamma: Gamma = generate_gamma(&syntax);
   
    // Transform all the interfaces 
    for trait_ in &gamma.traits {
        syntax.items.append(&mut transform_trait(&trait_, &gamma));
    }

    // Write output to file
    if write_and_fmt("outputs/output.rs", quote!(#syntax)).is_err() {
        panic!("Unable to write output file");
    }
}

fn main() {
    // print_goal();
    transform("./src/examples/generics/oop.rs");

    // examples::generics::fp::demo();
    // examples::generics::oop::demo();
}
