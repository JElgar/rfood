#![feature(rustc_private, box_patterns)]

extern crate syn;
extern crate proc_macro;

// extern crate rustc_ast;
// extern crate rustc_typeck;

use clap::Parser;
use rfood::transform::transformer::transform_file;
use rfood::cli::{Cli, Commands};

// use std::env;
use std::fs::File;
use std::io::Read;

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

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::PrintTest => print_goal(),
        Commands::Transform{path, output_path, transform_type} => transform_file(path, output_path, transform_type),
    }
}
