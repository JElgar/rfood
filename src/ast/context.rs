use std::collections::HashMap;
use syn::visit::{Visit, visit_item_enum, visit_item_trait, visit_item_struct, visit_item_impl};
use syn::{ItemEnum, ItemTrait, Variant, ItemStruct, Type, Ident};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[derive(Debug)]
pub struct Gamma {
    /// Enums are the datatypes
    pub enums: Vec<ItemEnum>, // DT - Datatypes
    /// Traits are the interfaces
    pub traits: Vec<ItemTrait>, // IT - Interfaces
    /// Enum variants are the constructors of a datatypes
    pub enum_variants: HashMap<ItemEnum, Punctuated<Variant, Comma>>, // CTR(DT) - Constructor for DT
    /// Generators are structs with an impl for a specific trait
    pub generators: HashMap<ItemTrait, Vec<ItemStruct>>, // GEN(IT) - Generic for IT

    // Helpers
    // All structs
    _structs: Vec<ItemStruct>,
}

impl Gamma {
    fn empty() -> Self {
        return Gamma {
            enums: Vec::new(),
            traits: Vec::new(),
            enum_variants: HashMap::new(),
            generators: HashMap::new(),
            _structs: Vec::new(),
        }
    }

    fn get_trait(&self, ident: &Ident) -> ItemTrait {
        self.traits.iter().find(|t| {
            t.ident == ident.clone()
        }).unwrap_or_else(|| panic!("Trait {} not found in gamma", ident)).clone()
    }
    
    fn get_struct(&self, ident: &Ident) -> ItemStruct {
        self._structs.iter().find(|s| {
            return s.ident == *ident 
        }).unwrap_or_else(|| panic!("Struct {} not found in gamma", ident)).clone()
    }
}

impl<'ast> Visit<'ast> for Gamma {
    fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
        visit_item_enum(self, i);
        self.enums.push(i.clone());
        self.enum_variants.insert(i.clone(), i.variants.clone());
    }

    fn visit_item_trait(&mut self, i: &'ast ItemTrait) {
        visit_item_trait(self, i);
        self.traits.push(i.clone());
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
        let struct_ = self.get_struct(&struct_name);
     
        // If the generator doesnt have any generators yet add an empty list
        if !self.generators.contains_key(&trait_) {
            self.generators.insert(trait_.clone(), Vec::new());
        }
        // Push the struct to the traits generator list
        self.generators.get_mut(&trait_).unwrap().push(struct_.clone());
    }
}

pub fn generate_gamma(syntax: &syn::File) -> Gamma {
    let mut gamma = Gamma::empty();
    gamma.visit_file(syntax);
    gamma
}
