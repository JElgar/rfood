extern crate proc_macro;

use std::collections::HashMap;
use syn::visit::{Visit, visit_item_enum, visit_item_trait, visit_item_struct, visit_item_impl};
use syn::*;
use crate::context::*;
use crate::ast::create::generic_parameter_from_generic_argument;
use errors::*;

pub fn get_generics_from_type(type_: &Type) -> Generics {
    if let Type::Path(TypePath{
        path: Path{
            segments,
            ..
        },
        ..
    }) = type_ {
        return get_generics_from_path_segment(segments.first().unwrap());
    }

    panic!("Not implemented. Cannot get generics from type.");
}

pub fn create_generics_from_args(args: &AngleBracketedGenericArguments) -> Generics {
    let mut generics = Generics::default();
    for arg in &args.args {
        generics.params.push(generic_parameter_from_generic_argument(arg));
    }
    return generics;
}

pub fn get_generics_from_path_segment(segment: &PathSegment) -> Generics {
    if let PathArguments::AngleBracketed(args) = &segment.arguments {
        return create_generics_from_args(args);
    }
    if let PathArguments::None = &segment.arguments {
        return Generics::default();
    }

    panic!("Cannot get generics from unsupported path segment, {:?}", segment);
}

/// Global context
#[derive(Debug, Clone)]
pub struct Gamma {
    /// Enums are the datatypes
    pub enums: Vec<ItemEnum>, // DT - Datatypes
    /// Traits are the interfaces
    pub traits: Vec<ItemTrait>, // IT - Interfaces
    /// Enum variants are the constructors of a datatypes
    pub enum_variants: HashMap<ItemEnum, Vec<Variant>>, // CTR(DT) - Constructor for DT
    /// Generators are structs with an impl for a specific trait, this stores both the struct and
    /// the impl
    pub generators: HashMap<ItemTrait, Vec<(ItemStruct, ItemImpl)>>, // GEN(IT) - Generic for IT
    /// Destructor of an interface - A function in a trait
    pub destructors: HashMap<ItemTrait, Vec<TraitItemMethod>>, // DTR(IT) - Destructor of IT
    /// Consumers of an enum (datatype) - A function that takes in a DT and return some kind of
    /// match on it
    // TODO: Collect
    pub enum_consumers: HashMap<ItemEnum, Vec<ItemStruct>>, // CSM(DT) - Consumer of DT

    // This is replaced with .signature
    // pub signatures: HashMap<Ident, Type>, // SIG(F) - Signature of F

    // Helpers
    /// All structs found in the ast -> Note these may not be inscope!
    _structs: Vec<ItemStruct>,

    pub functions: Vec<ItemFn>, // F - Functions, top level functions to transform
}

impl Gamma {
    fn empty() -> Self {
        return Gamma {
            enums: Vec::new(),
            traits: Vec::new(),
            enum_variants: HashMap::new(),
            generators: HashMap::new(),
            destructors: HashMap::new(),
            enum_consumers: HashMap::new(),

            _structs: Vec::new(),
            functions: Vec::new(),
        }
    }

    fn from_file(syntax: &syn::File) -> Self {
        let mut gamma = Gamma::empty();
        gamma.visit_file(syntax);
        gamma
    }

    pub fn is_trait(&self, ident: &Ident) -> bool {
        return self.get_trait(ident).is_ok();
    }

    pub fn get_trait(&self, ident: &Ident) -> std::result::Result<ItemTrait, NotFound> {
        match self.traits.iter().find(|t| {
            t.ident == ident.clone()
        }) {
            Some(t) => Ok(t.clone()),
            None => Err(NotFound{item_name: ident.to_string(), type_name: "trait".to_string()}),
        }
    }
    
    pub fn get_generators(&self, trait_: &ItemTrait) -> Vec<(ItemStruct, ItemImpl)> {
        self.generators.get(&trait_).unwrap_or_else(|| panic!("Trait {:?} not found in gamma", trait_)).clone()
    }

    pub fn get_all_generators(&self) -> Vec<(ItemStruct, ItemImpl)> {
        self.generators.iter().flat_map(|(_, v)| v.clone()).collect()
    }

    /// Check if the type is a generator, this can either be a struct or a trait name
    pub fn is_generator_type(&self, type_ident: &Ident) -> bool {
        self.get_all_generators().iter().any(|(struct_, _)| struct_.ident == *type_ident)
            || self.get_trait(type_ident).is_ok()
    }
    
    pub fn get_struct_by_name(&self, ident: &Ident) -> ItemStruct {
        self._structs.iter().find(|s| {
            return s.ident == *ident 
        }).unwrap().clone()
    }

    pub fn get_destructors(&self, trait_: &ItemTrait) -> Vec<TraitItemMethod> {
        self.destructors.get(&trait_).unwrap_or_else(|| panic!("Trait {:?} not found in gamma", trait_)).clone()
    }

    pub fn get_generator(&self, generator_ident: &Ident) -> ItemTrait {
        self.generators.iter().find_map(|(trait_, generators)| {
            return generators.iter().find_map(|(generator_struct, _)| {
                generator_struct.ident == *generator_ident;
                Some(trait_.clone())
            })
        }).unwrap()
    }
    
    pub fn get_destructor_signature(&self, generator_ident: &Ident, destructor_ident: &Ident) -> Signature {
        let traits = self.get_traits_for_generator(&generator_ident);
        self.traits.iter().find_map(|trait_| {
            self.get_destructors(&trait_).iter().find_map(|trait_item_method| {
                if trait_item_method.sig.ident == *destructor_ident {
                    return Some(
                        trait_item_method.sig.clone()
                    );
                }
                return None;
            })
        }).unwrap()
    }

    pub fn get_destructor_impl_for_generator(generator_impl: &ItemImpl, destructor_ident: &Ident) -> ImplItemMethod {
        // Filter all methods in the impl to find the one that matches the destructor
        generator_impl.items.iter().find_map(|item| {
            return match &*item {
                ImplItem::Method(impl_item_method) if impl_item_method.sig.ident == *destructor_ident => Some(impl_item_method.clone()),
                _ => None
            }
        })
        // If not found raise an exception
        .unwrap_or_else(|| panic!("Method {:?} not found in impl {:?}", destructor_ident, generator_impl))
    }

    pub fn is_interface(&self, ident: &Ident) -> bool {
        if ident == "Self" {
            return true;
        }
        self.traits.iter().find(|generator| generator.ident == *ident).is_some()
    }

    /// This method returns all the traits a struct implements
    ///
    /// This means if a struct implements 2 traits, it will return both of them
    pub fn get_traits_for_generator(&self, generator_ident: &Ident) -> Vec<ItemTrait> {
        Vec::from_iter(self.traits.iter().filter(|trait_| {
            self.get_generators(trait_).iter().any(|(generator_struct, _)| generator_struct.ident == *generator_ident)
        }).cloned())
    }
}

impl<'ast> Visit<'ast> for Gamma {
    fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
        visit_item_enum(self, i);
        self.enums.push(i.clone());
        self.enum_variants.insert(i.clone(), Vec::from_iter(i.variants.clone()));
    }

    fn visit_item_trait(&mut self, i: &'ast ItemTrait) {
        visit_item_trait(self, i);
        self.traits.push(i.clone());

        // Filter all the items in the trait and pull out the methods
        let trait_methods = Vec::from_iter(i.items.iter().filter_map(
            |item| {
                if let TraitItem::Method(impl_item_method) = item {
                    return Some(impl_item_method.clone());
                };
                return None
            }
          ));
        self.destructors.insert(i.clone(), trait_methods);
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        visit_item_struct(self, i);
        self._structs.push(i.clone());
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        visit_item_impl(self, i);

        // Find the trait that is being implemented
        let trait_ = self.get_trait(
            &i.trait_.as_ref().unwrap().1.segments.first().unwrap().ident
        ).unwrap();

        // Find the struct that the impl is for
        let struct_name: &Ident = if let Type::Path(type_path) = &*i.self_ty {
            &type_path.path.segments.first().unwrap().ident
        } else {
            panic!("Not a path when visiting item_impl");
        };
        let struct_ = self.get_struct_by_name(&struct_name);
     
        // If the generator doesnt have any generators yet add an empty list
        if !self.generators.contains_key(&trait_) {
            self.generators.insert(trait_.clone(), Vec::new());
        }

        // Push the struct to the traits generator list
        self.generators.get_mut(&trait_).unwrap().push((struct_.clone(), i.clone()));
    }

    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        self.functions.push(i.clone());
    }
}

pub fn generate_gamma(syntax: &syn::File) -> Gamma {
    Gamma::from_file(syntax)
}
