extern crate syn;

// use std::env;
use std::fs::File;
use std::io::Read;

fn get_traits(syntax: &syn::File) {
    println!("{:#?}", syntax);

    let traits: Vec<&syn::Item>;
    for item in &syntax.items {
        match item {
            syn::Item::Trait(syn::ItemTrait{
                ident,
                ..
            }) => {
                println!("Trait: ");
                println!("{:?}", item);
                // TODO Dont hard code this
                let vals = get_impls(syntax, ident.to_string());
                println!("{:?}", vals);
            }
            // syn::Item::Impl(_) => {
            //     println!("Impl: ");
            //     println!("{:?}", item);
            // }
            _ => (),
        }
    }
}

fn get_impls(syntax: &syn::File, trait_name: String) -> Vec<&syn::Item> {
    Vec::from_iter(syntax.items.iter().filter(
        |item| match item {
            syn::Item::Impl(syn::ItemImpl{
                trait_: Some(
                    (
                        _,
                        syn::Path{
                            segments,
                            ..
                        },
                        _
                    )
                ),
                ..
            }) if match segments.first() {
                Some(syn::PathSegment{
                    ident,
                    ..
                }) if ident.to_string() == trait_name => true,
                _ => false,
            } => true,
            _ => false,
        }
    ))
}

fn main() {
    // let mut args = env::args();
    // let _ = args.next(); // executable name

    let filename = "./src/oop/exp.rs";
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    get_traits(&syntax);
}
