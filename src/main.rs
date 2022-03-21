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
use std::path::PathBuf;

use clap::Parser;

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
  println!("{:#?}\n\n", syntax);
}

fn transform(path: &PathBuf) {
    //-- Do the transfrom --//
    let mut file = File::open(path).expect("Unable to open file");

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

#[derive(Parser)]
#[clap(name = "git")]
#[clap(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser)]
#[clap(name = "rfood")]
#[clap(bin_name = "rfood")]
enum Commands {
    #[clap()]
    PrintTest,
    #[clap(arg_required_else_help = true)]
    Transform{
        /// The path of the file to transform
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,
    },
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct PrintTest {}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct Transform {
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::PrintTest => print_goal(),
        Commands::Transform{path} => transform(path),
    }
}
