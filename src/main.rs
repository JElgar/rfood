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

use ast::print::write_and_fmt;

/// Struct to hold the info for a trait
#[derive(Debug)]
struct Trait {
    /// Name of the trait
    name: String,
    /// The implementations of the trait
    impls: Vec<Impl>,
    /// The methods in the trait
    methods: Vec<syn::TraitItemMethod>,
}

/// Struct to hold the info for an impl
#[derive(Debug)]
struct Impl {
    /// The name of the trait 
    name: String,
    attrs: Vec<syn::Attribute>,
    /// The required methods of the trait 
    methods: Vec<syn::ImplItemMethod>,
}

/// Struct to hold the info for a struct 
#[derive(Debug)]
struct Struct {
    /// The name of the struct
    name: String,
    /// The attributes of the struct
    fields: syn::Fields,
}

/// Converts a syn ItemImpl into a `Impl` struct
///
/// * `impl_` - The ItemImpl from syn, contains the data for an impl block
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

/// Convert trait signature to functional method signature
///
/// Replace &self with Box<T> and replace self with T
///
/// * `signature` - The signature of the trait method
/// * `name` - The name of the trait
///
/// Returns the function signature and the name of the type which replaces self if self is present
fn trait_signature_to_fp_function_signature(signature: syn::Signature, self_name: &String) -> (syn::Signature, Option<syn::Ident>) {
    println!("Current sig is: {:?}", signature);

    let colon = syn::token::Colon{
        spans: [syn::__private::Span::call_site()],
    };

    let mut self_replacement: Option<syn::Ident> = None;

    let new_inputs = syn::punctuated::Punctuated::from_iter(signature.inputs.iter().map(|item| {
      if let syn::FnArg::Receiver(arg_data) = item {
          self_replacement = Some(syn::Ident::new(&self_name.to_lowercase(), syn::__private::Span::call_site()));
          return syn::FnArg::Typed(
              syn::PatType{
                  attrs: arg_data.attrs.clone(),
                  colon_token: colon,
                  pat: Box::new(
                      syn::Pat::Ident(syn::PatIdent{
                          attrs: [].to_vec(),
                          by_ref: None,
                          mutability: None,
                          // TODO make this not just exp
                          ident: self_replacement.clone().unwrap(),
                          subpat: None,
                      })
                  ),
                  ty: Box::new(syn::Type::Reference(
                    syn::TypeReference{
                        and_token: syn::token::And { spans: [syn::__private::Span::call_site()] },
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(
                            syn::Type::Path(
                                syn::TypePath{
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: syn::punctuated::Punctuated::from_iter(
                                            vec![
                                              syn::PathSegment{
                                                ident: syn::Ident::new(self_name, syn::__private::Span::call_site()),
                                                arguments: syn::PathArguments::None,
                                              }
                                            ]
                                        )
                                    }
                                }
                            )
                        ),
                    }
                  ))
              }
          )
      }
      item.clone()
    }));

    (
        syn::Signature {
            inputs: new_inputs,
            ..signature
        },  
        self_replacement
    )
}

/// Given a trait method find the matching impl method
fn get_matching_impl_method(trait_method: &syn::TraitItemMethod, impl_: &Impl) -> syn::ImplItemMethod {
    return impl_.methods.iter().find_map(|method| {
        if method.sig == trait_method.sig {
            return Some(method.clone());
        }
        return None
    }).unwrap_or_else(|| panic!("Could not find matching method"));
}

/// Given a syn::ImplItemMethod, extract the expression
fn get_impl_method_expression(method: syn::ImplItemMethod) -> syn::Expr {
    match method.block.stmts.first() {
        Some(
            syn::Stmt::Semi(syn::Expr::Return(syn::ExprReturn{expr: Some(expr), ..}), _)
        ) => *expr.clone(),
        _ => panic!("Could not find expression in method")
    }
}

/// Given the syntax find all traits
///
/// * `syntax` - The syntax tree of the input file 
fn get_traits(syntax: &syn::File) -> Vec<Trait> {
    let mut traits: Vec<Trait> = Vec::new();
    for item in &syntax.items {
        if let syn::Item::Trait(trait_data) = item {
          traits.push(Trait{
            name: trait_data.ident.to_string(),
            impls: get_impls(syntax, trait_data.ident.to_string()),
            methods: Vec::from_iter(trait_data.items.iter().filter_map(
              |item| {
                  if let syn::TraitItem::Method(impl_item_method) = item {
                      return Some(impl_item_method.clone());
                  };
                  return None
              }
            )),
          });
        }
    }
    return traits;
}

/// Given the syntax find the first enum 
///
/// * `syntax` - The syntax tree of the input file 
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


/// Given the syntax find all impls for a given trait
///
/// * `syntax` - The syntax tree of the input file 
/// * `trait_name` - The name of the trait to find impls for
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

/// Find a struct is the syntax 
///
/// * `syntax` - The syntax tree of the input file 
/// * `struct_name` - The name of the struct to find
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

fn print_goal() {
  // -- Print current and goal enum --//
  let filename = "./src/test.rs";
  let mut file = File::open(&filename).expect("Unable to open file");

  let mut src = String::new();
  file.read_to_string(&mut src).expect("Unable to read file");

  let syntax: syn::File = syn::parse_file(&src).expect("Unable to parse file");
  println!("{:?}", syntax);
}

fn main() {
    print_goal();
    println!();
    println!();
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
            variants.push(ast::create::create_enum_variant(&variant.name, impl_struct.fields));
        }

        // For each method in the trait find the matching implementation in each impl
        for method in &trait_.methods {
            println!("Method: {:?}", method);
            for impl_ in &trait_.impls {
                println!("Matching method for impl {:?}: {:?}", impl_.name, get_matching_impl_method(method, impl_));
            }
            
            let (fp_signature, self_indent) = trait_signature_to_fp_function_signature(method.sig.clone(), &trait_.name);

            let mut function_stmts: Vec<syn::Stmt> = Vec::new();

            // Create a match statement which matches on the enum and uses the method
            match self_indent {
                Some(ident) => {
                    let mut arms: Vec<syn::Arm> = Vec::new();
                    // For each impl create a match arm
                    for impl_ in &trait_.impls {
                        let expr: syn::Expr = get_impl_method_expression(get_matching_impl_method(method, impl_));
                        let path = ast::create::create_match_path_for_enum(&trait_.name, &impl_.name);
                        let match_arm = ast::create::create_match_arm(
                            path, Vec::new(), expr,
                        );
                        arms.push(match_arm);
                    }


                    let match_expr = ast::create::create_match_statement(ident, arms);
                    function_stmts.push(
                        syn::Stmt::Expr(match_expr)
                    );
                },
                None => {panic!("Cannot yet transform method with no self")}
            }
            
            // Create a method with the functional programming function signature
            let function = ast::create::create_function(fp_signature, function_stmts);
            syntax.items.push(function);
        }

        let new_enum: syn::Item = ast::create::create_enum(&trait_.name, variants);
        syntax.items.push(new_enum);
    }

    // TODO: https://stackoverflow.com/questions/65764987/how-to-pretty-print-syn-ast
    println!("{}", quote!(#syntax));

    if write_and_fmt("outputs/output.rs", quote!(#syntax)).is_err() {
        panic!("Unable to write output file");
    }
}
