#![feature(rustc_private, box_patterns)]

extern crate syn;
extern crate proc_macro;
#[macro_use]
extern crate quote;

// extern crate rustc_ast;
// extern crate rustc_typeck;

use rfood::*;

// use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use clap::Parser;

use ast::print::write_and_fmt;
use rfood::context::gamma::{Gamma, generate_gamma};
use transform::transformer::{transform_trait, transform_enum, transform_item, TransformType};
use cli::{Cli, Commands};

fn print_goal() {
  // -- Print current and goal enum --//
  // let filename = "./src/examples/generics/oop.rs";
  let filename = "./src/test.rs";
  let mut file = File::open(&filename).expect("Unable to open file");

  let mut src = String::new();
  file.read_to_string(&mut src).expect("Unable to read file");

  let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
  println!("{:#?}\n\n", syntax);
}

fn remove_item_from_syntax(syntax: &mut syn::File, item: syn::Item) {
    let index = syntax.items.iter().position(|sitem| *sitem == item);
    if index.is_some() {
        syntax.items.remove(index.unwrap());
    }
}

fn transform(path: &PathBuf, transform_type: &TransformType) {
    //-- Do the transfrom --//
    let mut file = File::open(path).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    let mut syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    let mut transformed_syntax = syn::File{
        items: Vec::new(),
        ..syntax.clone()
    };

    // Generate global gamma context
    let mut gamma: Gamma = generate_gamma(&syntax);
    let gamma_mut_borrow = &mut gamma;
  
    match transform_type {
        TransformType::OOPToFP => {
            // Transform all the interfaces 
            for trait_ in gamma_mut_borrow.traits.clone() {
                // Add the transformed items to the transformed syntax
                transformed_syntax.items.append(&mut transform_trait(&trait_, gamma_mut_borrow));

                // Remove the original trait from the syntax
                for (item_struct, item_impl) in gamma_mut_borrow.get_generators(&trait_) {
                    remove_item_from_syntax(&mut syntax, syn::Item::Struct(item_struct));
                    remove_item_from_syntax(&mut syntax, syn::Item::Impl(item_impl));
                }
                remove_item_from_syntax(&mut syntax, syn::Item::Trait(trait_.clone()));
            }
        }, 
        TransformType::FPToOOP => {
            // Transform all the enums
            for enum_ in gamma_mut_borrow.enums.clone() {
                // Get the consumers for the enum 
                let consumers = gamma_mut_borrow.get_enum_consumers(&enum_);
                for val in consumers.iter() {
                    print!("Creating a thing for the consumer: {:#?}\n", val.sig.ident);
                }

                // Create a trait 
                transformed_syntax.items.extend(transform_enum(&enum_, gamma_mut_borrow));

                // For all the consumers, for each arm create a method in each impl
                for consumer in consumers {
                    remove_item_from_syntax(&mut syntax, syn::Item::Fn(consumer.clone()));
                }
                remove_item_from_syntax(&mut syntax, syn::Item::Enum(enum_.clone()));
            }
            
            // Transform all the consumers
        }
    }

    for item in &syntax.items {
        transformed_syntax.items.push(transform_item(item, &transform_type, &gamma));
    }

    // Write output to file
    if write_and_fmt("outputs/output.rs", quote!(#transformed_syntax)).is_err() {
        panic!("Unable to write output file");
    }
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::PrintTest => print_goal(),
        Commands::Transform{path, transform_type} => transform(path, transform_type),
    }
}
