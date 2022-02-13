extern crate syn;
extern crate proc_macro;
#[macro_use]
extern crate quote;

// use std::env;
use std::fs::File;
use std::io::Read;

mod ast;
mod fp;

#[derive(Debug)]
struct Trait {
    name: String,
    impls: Vec<Impl>,
}

#[derive(Debug)]
struct Impl {
    name: String,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug)]
struct Struct {
    name: String,
    fields: syn::Fields,
}

fn get_traits(syntax: &syn::File) -> Vec<Trait> {
    // println!("{:#?}", syntax);
    let mut traits: Vec<Trait> = Vec::new();
    for item in &syntax.items {
        match item {
            syn::Item::Trait(syn::ItemTrait{
                ident,
                ..
            }) => {
                println!("Trait: {}", ident.to_string());
                println!("{:?}", item);
                let vals = get_impls(syntax, ident.to_string());
                println!("{:?}", vals);
                traits.push(Trait{
                    name: ident.to_string(),
                    impls: vals,
                });
            }
            // syn::Item::Impl(_) => {
            //     println!("Impl: ");
            //     println!("{:?}", item);
            // }
            _ => (),
        }
    }
    return traits;
}

fn get_enum(syntax: &syn::File) -> syn::ItemEnum {
    // println!("{:#?}", syntax);
    let enum_ = syntax.items.iter().find_map(
        |item| match item {
            syn::Item::Enum(item_enum) => return Some(item_enum),
            _ => None
        }
    );

    if let Some(v) = enum_ {
        return v.clone()
    }

    panic!("No enums found in file")
}


fn get_impls(syntax: &syn::File, trait_name: String) -> Vec<Impl> {
    let vals = syntax.items.iter().filter(
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
    );
    println!("{:?}", Vec::from_iter(vals.clone()));
    Vec::from_iter(vals.map(
        |item| match item {
            syn::Item::Impl(syn::ItemImpl{
                attrs,
                self_ty,
                ..
            }) => {
                match &**self_ty {
                    syn::Type::Path(
                        syn::TypePath{
                            path: syn::Path{
                                segments,
                                ..
                            },
                            ..
                        }
                    ) if segments.first().is_some() => Impl{
                        name: segments.first().unwrap().ident.to_string(),
                        attrs: attrs.clone(),
                    },
                    _ => panic!("Could not find name of impl")
                }
            }
            _ => panic!("An impl value is not a valid impl")
        }
    ))
}

fn get_struct(syntax: &syn::File, struct_name: &String) -> Struct {
    match syntax.items.iter().find_map(
        |item| match item {
            syn::Item::Struct(syn::ItemStruct{
                ident,
                fields,
                ..
            }) if ident.to_string() == *struct_name => {
                println!("\n\n");
                println!("{}", "The struct");
                println!("{:?}", item);
                println!("\n\n");
                Some(Struct{name: ident.to_string(), fields: fields.clone()})
            },
            _ => None,
        }
    ) {
        Some(v) => v,
        _ => panic!("Could not find struct with name {}", struct_name),
    }
}

fn main() {
    // let mut args = env::args();
    // let _ = args.next(); // executable name
    
    // let filename = "./src/fp/set.rs";
    // let mut file = File::open(&filename).expect("Unable to open file");
    // let mut src = String::new();
    // file.read_to_string(&mut src).expect("Unable to read file");
    // let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    // let enum_ = get_enum(&syntax);
    // println!("{:?}", enum_);

    //-- Do the transfrom --//
    let filename = "./src/oop/set.rs";
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    let mut syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    let traits = get_traits(&syntax);

    for trait_ in &traits {
        let mut variants: Vec<syn::Variant> = Vec::new();

        for variant in &trait_.impls {
            let impl_struct = get_struct(&syntax, &variant.name);
            println!("Varaint: {}", variant.name);
            println!("Struct: {:?}", impl_struct);
            // variants.push(ast::create::create_enum_variant(&variant.name, ast::create::create_enum_unnamed_fields(Vec::new())));
            variants.push(ast::create::create_enum_variant(&variant.name, impl_struct.fields));
        }

        let new_enum: syn::Item = ast::create::create_enum(&trait_.name, variants);
        syntax.items.push(new_enum);
    }
   
    // Get the generated enum
    let enum_ = get_enum(&syntax);
    println!("{:?}", enum_);

    println!("{}", quote!(#syntax))
   
    //-- Print current and goal enum --//
    // let filename = "./src/test.rs";
    // let mut file = File::open(&filename).expect("Unable to open file");

    // let mut src = String::new();
    // file.read_to_string(&mut src).expect("Unable to read file");

    // let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    // println!("Current enum: ");
    // println!("{:?}", syntax.items[0]);
    // println!("\n\n");
    // println!("Required enum: ");
    // println!("{:?}", syntax.items[1]);
}
