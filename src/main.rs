extern crate syn;
extern crate proc_macro;
#[macro_use]
extern crate quote;

// use std::env;
use std::fs::File;
use std::io::Read;

mod ast;
mod fp;
mod oop;

#[derive(Debug)]
struct Trait {
    name: String,
    impls: Vec<Impl>,
}

#[derive(Debug)]
struct Impl {
    name: String,
    attrs: Vec<syn::Attribute>,
    methods: Vec<syn::ImplItemMethod>,
}

#[derive(Debug)]
struct Struct {
    name: String,
    fields: syn::Fields,
}

fn syn_impl_to_impl(impl_: &syn::ItemImpl) -> Impl {
  match &*impl_.self_ty {
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
        attrs: impl_.attrs.clone(),
        methods: Vec::from_iter(impl_.items.iter().filter_map(
            |item| {
                if let syn::ImplItem::Method(impl_item_method) = item {
                    return Some(impl_item_method.clone());
                };
                return None
            }
        )),
    },
    _ => panic!("Could not find name of impl")
  }
}

fn get_traits(syntax: &syn::File) -> Vec<Trait> {
    let mut traits: Vec<Trait> = Vec::new();
    for item in &syntax.items {
        if let syn::Item::Trait(trait_data) = item {
          traits.push(Trait{
              name: trait_data.ident.to_string(),
              impls: get_impls(syntax, trait_data.ident.to_string()),
          });
        }
    }
    return traits;
}

fn get_enum(syntax: &syn::File) -> syn::ItemEnum {
    let enum_ = syntax.items.iter().find_map(
        |item| match item {
            syn::Item::Enum(item_enum) => return Some(item_enum),
            _ => None
        }
    );

    if enum_.is_none() {
        panic!("No enums found in file")
    }

    enum_.unwrap().clone()
}


fn get_impls(syntax: &syn::File, trait_name: String) -> Vec<Impl> {
    // Filter all impls for the given trait and map them to a Impl struct
    Vec::from_iter(syntax.items.iter().filter_map(
        |item| {
            if let syn::Item::Impl(item_data) = item {
                if let Some(
                    (
                        _,
                        syn::Path{
                            segments,
                            ..
                        },
                        _
                    )
                ) = &item_data.trait_ {
                    if let Some(syn::PathSegment{ident, ..}) = segments.first() {
                        if ident.to_string() == trait_name {
                            return Some(syn_impl_to_impl(item_data));
                        }
                    }
                }
            }
            return None
        }
    ))
}

fn get_struct(syntax: &syn::File, struct_name: &String) -> Struct {
    let struct_ = syntax.items.iter().find_map(
        |item| match item {
            syn::Item::Struct(syn::ItemStruct{
                ident,
                fields,
                ..
            }) if ident.to_string() == *struct_name => {
                Some(Struct{name: ident.to_string(), fields: fields.clone()})
            },
            _ => None,
        }
    );

    if struct_.is_none() {
        panic!("Could not find struct with name {}", struct_name)
    }
    return struct_.unwrap();
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
    
    // fp::set::demo();

    //-- Do the transfrom --//
    let filename = "./src/oop/exp.rs";
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    let mut syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
    let traits = get_traits(&syntax);

    // Create enum from trait and its impls
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
    
    // Create function for each trait function using the impls
    //
   
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
