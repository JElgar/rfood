extern crate proc_macro;

use std::collections::HashMap;
use syn::visit::{Visit, visit_item_enum, visit_item_trait, visit_item_struct, visit_item_impl};
use syn::{ItemEnum, ItemTrait, Variant, ItemStruct, Type, Ident, TraitItem, TraitItemMethod, ImplItemMethod, ItemImpl, ImplItem, Expr};

trait GammaExpr {
    fn get_signature(&self) -> (Type, Vec<Type>);
}

/// Global context
#[derive(Debug)]
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
        }
    }

    fn from_file(syntax: &syn::File) -> Self {
        let mut gamma = Gamma::empty();
        gamma.visit_file(syntax);
        gamma
    }

    pub fn get_trait(&self, ident: &Ident) -> ItemTrait {
        self.traits.iter().find(|t| {
            t.ident == ident.clone()
        }).unwrap_or_else(|| panic!("Trait {} not found in gamma", ident)).clone()
    }
    
    pub fn get_generators(&self, trait_: &ItemTrait) -> Vec<(ItemStruct, ItemImpl)> {
        self.generators.get(&trait_).unwrap_or_else(|| panic!("Trait {:?} not found in gamma", trait_)).clone()
    }
    
    pub fn get_struct_by_name(&self, ident: &Ident) -> ItemStruct {
        self._structs.iter().find(|s| {
            return s.ident == *ident 
        }).unwrap().clone()
    }

    pub fn get_destructors(&self, trait_: &ItemTrait) -> Vec<TraitItemMethod> {
        self.destructors.get(&trait_).unwrap_or_else(|| panic!("Trait {:?} not found in gamma", trait_)).clone()
    }

    pub fn get_destructor_impl_for_generator(generator_impl: &ItemImpl, destructor: &TraitItemMethod) -> ImplItemMethod {
        // Filter all methods in the impl to find the one that matches the destructor
        generator_impl.items.iter().find_map(|item| {
            return match &*item {
                ImplItem::Method(impl_item_method) if impl_item_method.sig.ident == destructor.sig.ident => Some(impl_item_method.clone()),
                _ => None
            }
        })
        // If not found raise an exception
        .unwrap_or_else(|| panic!("Method {:?} not found in impl {:?}", destructor, generator_impl))
    }

    pub fn is_interface(&self, ident: &Ident) -> bool {
        if ident == "Self" {
            return true;
        }
        self.traits.iter().find(|generator| generator.ident == *ident).is_some()
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
        );

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
}

pub fn generate_gamma(syntax: &syn::File) -> Gamma {
    Gamma::from_file(syntax)
}
