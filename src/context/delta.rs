// Extending the implementation to use THIR/MIR
// https://rustc-dev-guide.rust-lang.org/thir.html
// https://rustc-dev-guide.rust-lang.org/the-parser.html 


extern crate rustc_typeck;
extern crate rustc_middle;
extern crate rustc_interface;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

use std::path::Path;
use std::collections::HashMap;

use rustc_middle::ty;
use rustc_span::source_map::FilePathMapping;

use rustc_session::parse::ParseSess;

pub struct Delta {
    types: HashMap<Ident, Type>,
}

use syn::{Item, ItemEnum, ItemTrait, Variant, ItemStruct, Type, Ident, TraitItem, TraitItemMethod};
use rustc_middle::ty::{TyCtxt};

trait Transformable {
    fn generate_delta(&self) -> Delta;
    fn transform(&self, delta: Delta) -> Self;
}

impl Delta {
    pub fn generate_for_file() {
        // ty::TyCtxt::create_global_ctxt();
        // ty::TyCtxt::create_and_enter(hir, ..., |tcx| {
        //     rustc_typeck::check_crate(tcx).unwrap();
        // });
        
        // let config = rustc_interface::interface::Config{};
        // rustc_interface::interface::run_compiler(config, |compiler| {
        //     println!("hello world");
        // });

        println!("Generating delta for file");
        let path = Path::new("/home/jelgar/Documents/uni/project/hello_world/");
        let sess = ParseSess::new(FilePathMapping::empty());
        println!("Generated session");
        let result = rustc_parse::parse_crate_from_file(path, &sess);
        println!("{:?}", result);
    }
}
