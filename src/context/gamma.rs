extern crate proc_macro;

use crate::ast::create::generic_parameter_from_generic_argument;
use crate::context::delta::{GetDeltaType, DeltaType};
use crate::context::*;
use crate::transform::transformer::TransformType;
use errors::*;
use std::collections::HashMap;
use syn::visit::{visit_item_enum, visit_item_impl, visit_item_struct, visit_item_trait, Visit};
use syn::*;

pub fn get_generics_from_type(type_: &Type) -> Generics {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = type_
    {
        return get_generics_from_path_segment(segments.first().unwrap());
    }

    panic!("Not implemented. Cannot get generics from type.");
}

pub fn get_fn_arg_name(fn_arg: &FnArg) -> Ident {
    if let FnArg::Typed(PatType { box pat, .. }) = fn_arg {
        if let Pat::Ident(PatIdent { ident, .. }) = pat {
            return ident.clone();
        }
    }

    panic!("Not implemented. Cannot get fn arg name. {:?}", fn_arg);
}

pub fn create_generics_from_args(args: &AngleBracketedGenericArguments) -> Generics {
    let mut generics = Generics::default();
    for arg in &args.args {
        generics
            .params
            .push(generic_parameter_from_generic_argument(arg));
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

    panic!(
        "Cannot get generics from unsupported path segment, {:?}",
        segment
    );
}

pub fn get_consumer_match_statement(consumer: &ItemFn) -> std::result::Result<ExprMatch, NotFound> {
    let last_stmt = consumer.block.stmts.last();
    if let Some(stmt) = last_stmt {
        match stmt {
            Stmt::Expr(Expr::Match(expr_match))
            | Stmt::Expr(Expr::Return(ExprReturn {
                expr: Some(box Expr::Match(expr_match)),
                ..
            }))
            | Stmt::Semi(
                Expr::Return(ExprReturn {
                    expr: Some(box Expr::Match(expr_match)),
                    ..
                }),
                _,
            ) => {
                return Ok(expr_match.clone());
            }
            _ => (),
        }
    }
    return Err(NotFound {
        item_name: "Consumer match statement".to_string(),
        type_name: "ExprMatch".to_string(),
    });
}

pub fn get_match_expr_for_enum(
    consumer: &ItemFn,
    enum_variant_ident: &Ident,
) -> std::result::Result<Expr, NotFound> {
    let match_expr = get_consumer_match_statement(consumer);
    if match_expr.is_err() {
        return Err(match_expr.err().unwrap());
    }

    let enum_expr = match_expr.clone().unwrap().arms.iter().find_map(|arm| {
        // If the arm pat is the enum
        if let Pat::Struct(PatStruct {
            path: Path { segments, .. },
            ..
        }) = &arm.pat
        {
            if segments.last().unwrap().ident == *enum_variant_ident {
                return Some(*arm.body.clone());
            }
        }
        return None;
    });

    // If there is an arm for the enum, return that
    if enum_expr.is_some() {
        return Ok(enum_expr.unwrap());
    }

    // Otherwise get the wildcard arm
    Ok(match_expr
        .unwrap()
        .arms
        .iter()
        .find_map(|arm| {
            if let Pat::Wild(_) = &arm.pat {
                return Some(*arm.body.clone());
            }
            return None;
        })
        .unwrap())
}

pub fn is_mutable_self(sig: &Signature) -> bool {
    match sig.inputs.first() {
        Some(FnArg::Receiver(
            Receiver { 
                mutability: Some(_),
                self_token: token::SelfValue{..}, 
                ..
            }
        )) => true,
        _ => false,
    }
}

/// Global context
#[derive(Debug, Clone)]
pub struct Gamma {
    /// Enums are the datatypes
    pub enums: Vec<ItemEnum>, // DT - Datatypes
    /// Traits are the interfaces
    pub traits: Vec<ItemTrait>, // IT - Interfaces
    /// Generators are structs with an impl for a specific trait, this stores both the struct and
    /// the impl
    // The first ident is the ident of the ItemTrait
    pub generators: HashMap<Ident, Vec<(ItemStruct, ItemImpl)>>, // GEN(IT) - Generic for IT
    /// Destructor of an interface - A function in a trait
    // The first ident is the ident of the ItemTrait
    pub destructors: HashMap<Ident, Vec<TraitItemMethod>>, // DTR(IT) - Destructor of IT
    /// Consumers of an enum (datatype) - A function that takes in a DT and return some kind of
    /// match on it. This stores the enum and all the itemfns
    // The first ident is the ident of the ItemEnum 
    pub enum_consumers: HashMap<Ident, HashMap<Ident, ItemFn>>, // CSM(DT) - Consumer of DT

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
            generators: HashMap::new(),
            destructors: HashMap::new(),
            enum_consumers: HashMap::new(),

            _structs: Vec::new(),
            functions: Vec::new(),
        };
    }

    fn from_file(syntax: &syn::File) -> Self {
        let mut gamma = Gamma::empty();
        gamma.visit_file(syntax);
        gamma
    }

    pub fn is_trait(&self, ident: &Ident) -> bool {
        return self.get_trait(ident).is_ok();
    }

    pub fn is_enum(&self, ident: &Ident) -> bool {
        return self.get_enum(ident).is_ok();
    }

    pub fn is_enum_or_variant(&self, ident: &Ident) -> bool {
        return self
            .enums
            .iter()
            .any(|enum_| enum_.variants.iter().any(|variant| variant.ident == *ident))
            || self.is_enum(ident);
    }
    
    pub fn get_trait(&self, ident: &Ident) -> std::result::Result<ItemTrait, NotFound> {
        match self.traits.iter().find(|t| t.ident == ident.clone()) {
            Some(t) => Ok(t.clone()),
            None => Err(NotFound {
                item_name: ident.to_string(),
                type_name: "trait".to_string(),
            }),
        }
    }

    pub fn get_enum(&self, ident: &Ident) -> std::result::Result<ItemEnum, NotFound> {
        match self.enums.iter().find(|e| e.ident == ident.clone()) {
            Some(e) => Ok(e.clone()),
            None => Err(NotFound {
                item_name: ident.to_string(),
                type_name: "enum".to_string(),
            }),
        }
    }

    pub fn get_enum_variant(&self, enum_ident: &Ident, enum_variant_ident: &Ident) -> Variant {
        let enum_ = self
            .get_enum(&self.get_base_type_name_from_type_name(enum_ident))
            .unwrap();
        enum_
            .variants
            .clone()
            .iter()
            .find(|v| v.ident == enum_variant_ident.clone())
            .unwrap()
            .clone()
    }

    pub fn get_generators(&self, trait_ident: &Ident) -> Vec<(ItemStruct, ItemImpl)> {
        self.generators
            .get(&trait_ident)
            .unwrap_or_else(|| panic!("Trait {:?} not found in gamma generators", trait_ident))
            .clone()
    }

    pub fn is_generator_of_trait(&self, trait_ident: &Ident) -> bool {
        self.get_generators(&trait_ident)
            .iter()
            .any(|(struct_, _)| struct_.ident == *trait_ident)
    }

    pub fn get_all_generators(&self) -> Vec<(ItemStruct, ItemImpl)> {
        self.generators
            .iter()
            .flat_map(|(_, v)| v.clone())
            .collect()
    }

    /// Check if the type is a generator, this can either be a struct or a trait name
    pub fn is_generator_type(&self, type_ident: &Ident) -> bool {
        self.get_all_generators()
            .iter()
            .any(|(struct_, _)| struct_.ident == *type_ident)
            || self.get_trait(type_ident).is_ok()
    }

    pub fn is_consumer(&self, fn_ident: &Ident) -> bool {
        self.enum_consumers
            .iter()
            .any(|(_, v)| v.values().any(|item_fn| item_fn.sig.ident == *fn_ident))
    }

    pub fn is_desturctor_of_trait(&self, trait_ident: &Ident, fn_ident: &Ident) -> bool {
        self.destructors.get(trait_ident).unwrap().iter().any(|item_fn| item_fn.sig.ident == *fn_ident)
    }

    /// For a given generator (struct) find the trait that it implements
    ///
    /// This could be a many to many relationship. For now we only return the first one and
    /// restrict the user to only have one trait per generator.
    pub fn get_generator_trait(&self, generator_ident: &Ident) -> Option<ItemTrait> {
        self.traits
            .iter()
            .find(|t| {
                self.get_generators(&t.ident)
                    .iter()
                    .any(|(struct_, _)| struct_.ident == *generator_ident)
            })
            .cloned()
    }

    pub fn get_struct_by_name(&self, ident: &Ident) -> ItemStruct {
        self._structs
            .iter()
            .find(|s| return s.ident == *ident)
            .unwrap_or_else(|| panic!("Struct {:?} not found in gamma", ident))
            .clone()
    }

    pub fn get_destructors(&self, trait_ident: &Ident) -> Vec<TraitItemMethod> {
        self.destructors
            .get(&trait_ident)
            .unwrap_or_else(|| panic!("Trait {:?} not found in gamma destructors the traits are: {:?}", trait_ident, self.traits))
            .clone()
    }

    pub fn get_generator(&self, generator_ident: &Ident) -> ItemTrait {
        self.generators
            .iter()
            .find_map(|(trait_ident, generators)| {
                return generators.iter().find_map(|(generator_struct, _)| {
                    generator_struct.ident == *generator_ident;
                    Some(self.get_trait(trait_ident).unwrap().clone())
                });
            })
            .unwrap()
    }

    pub fn get_signature(&self, fn_ident: &Ident) -> std::result::Result<Signature, NotFound> {
        match self.functions.iter().find(|f| f.sig.ident == *fn_ident) {
            Some(f) => Ok(f.sig.clone()),
            None => Err(NotFound {
                item_name: fn_ident.to_string(),
                type_name: "function".to_string(),
            }),
        }
    }

    pub fn get_destructor_signature(
        &self,
        generator_ident: &Ident,
        destructor_ident: &Ident,
    ) -> Signature {
        let traits = self.get_traits_for_generator(&generator_ident);
        self.traits
            .iter()
            .find_map(|trait_| {
                self.get_destructors(&trait_.ident)
                    .iter()
                    .find_map(|trait_item_method| {
                        if trait_item_method.sig.ident == *destructor_ident {
                            return Some(trait_item_method.sig.clone());
                        }
                        return None;
                    })
            })
            .unwrap()
    }

    pub fn get_destructor_impl_for_generator(
        generator_impl: &ItemImpl,
        destructor_ident: &Ident,
    ) -> std::result::Result<ImplItemMethod, NotFound> {
        // Filter all methods in the impl to find the one that matches the destructor
        match generator_impl.items.iter().find_map(|item| {
            return match &*item {
                ImplItem::Method(impl_item_method)
                    if impl_item_method.sig.ident == *destructor_ident =>
                {
                    Some(impl_item_method.clone())
                }
                _ => None,
            };
        }) {
            Some(impl_item_method) => Ok(impl_item_method),
            None => Err(NotFound {
                item_name: destructor_ident.to_string(),
                type_name: "destructor".to_string(),
            }),
        }
    }

    pub fn get_destructor_impl_for_trait(
        trait_: &ItemTrait,
        destructor_ident: &Ident,
    ) -> std::result::Result<TraitItemMethod, NotFound> {
        // Filter all methods in the impl to find the one that matches the destructor
        match trait_.items.iter().find_map(|item| {
            return match &*item {
                TraitItem::Method(impl_item_method)
                    if impl_item_method.sig.ident == *destructor_ident =>
                {
                    Some(impl_item_method.clone())
                }
                _ => None,
            };
        }) {
            Some(impl_item_method) => Ok(impl_item_method),
            None => Err(NotFound {
                item_name: destructor_ident.to_string(),
                type_name: "destructor".to_string(),
            }),
        }
    }

    pub fn is_interface(&self, ident: &Ident) -> bool {
        if ident == "Self" {
            return true;
        }
        self.traits
            .iter()
            .find(|generator| generator.ident == *ident)
            .is_some()
    }

    /// This method returns all the traits a struct implements
    ///
    /// This means if a struct implements 2 traits, it will return both of them
    pub fn get_traits_for_generator(&self, generator_ident: &Ident) -> Vec<ItemTrait> {
        Vec::from_iter(
            self.traits
                .iter()
                .filter(|trait_| {
                    self.get_generators(&trait_.ident)
                        .iter()
                        .any(|(generator_struct, _)| generator_struct.ident == *generator_ident)
                })
                .cloned(),
        )
    }

    pub fn add_enum(&mut self, enum_: &ItemEnum) {
        self.enums.push(enum_.clone());
    }

    pub fn add_enum_consumer(
        &mut self,
        enum_: &ItemEnum,
        consumer_ident: &Ident,
        consumer: &ItemFn,
    ) {
        self.enum_consumers
            .entry(enum_.ident.clone())
            .or_insert_with(HashMap::new)
            .insert(consumer_ident.clone(), consumer.clone());
        self.functions.push(consumer.clone());
    }

    pub fn add_trait(&mut self, trait_: &ItemTrait) {
        self.traits.push(trait_.clone());
        // Add the destructors
        for item in &trait_.items {
            if let TraitItem::Method(method)  = item {
                self.add_destructor(&trait_.ident, &method);
            }
        }
    }

    pub fn add_generator(
        &mut self,
        trait_: &ItemTrait,
        generator_struct: &ItemStruct,
        generator_impl: &ItemImpl,
    ) {
        self.generators
            .entry(trait_.ident.clone())
            .or_insert_with(Vec::new)
            .push((generator_struct.clone(), generator_impl.clone()));
        self.add_struct(generator_struct);
    }

    pub fn add_destructor(&mut self, trait_ident: &Ident, destructor: &TraitItemMethod) {
        self.destructors
            .entry(trait_ident.clone())
            .or_insert_with(Vec::new)
            .push(destructor.clone());
    }

    pub fn get_type_of_field(&self, struct_ident: &Ident, field_ident: &Ident) -> DeltaType {
        self._structs.iter().find(|struct_| struct_.ident == *struct_ident).unwrap().fields.iter().find(|field| {
            field.ident.as_ref().unwrap() == field_ident
        }).unwrap().ty.clone().get_delta_type()
    }

    pub fn add_struct(&mut self, struct_: &ItemStruct) {
        self._structs.push(struct_.clone());
    }

    /// Return true if the type is a subtype (enum variant of sturct that impl trait) of the super
    /// type.
    pub fn is_subtype_of(&self, type_name: &Ident, super_type_name: &Ident) -> bool {
        if self.is_trait(super_type_name) {
            return self.get_generators(&super_type_name).iter().any(|(struct_, _)| {
                struct_.ident == *type_name 
            });
        }

        if self.is_enum(super_type_name) {
            return self.get_enum(&super_type_name).unwrap().variants.iter().any(|variant| {
                variant.ident == *type_name 
            });
        }

        return false;
    }

    /// Given a struct/enum varaient name, get the base type name (trait/enum)
    pub fn get_base_type_name_from_type_name(&self, type_name: &Ident) -> Ident {
        if !self.is_trait(type_name) {
            self.get_generator_trait(type_name).unwrap().ident
        } else {
            type_name.clone()
        }
    }

    pub fn get_enum_consumers(&self, enum_: &ItemEnum) -> Vec<ItemFn> {
        if self.enum_consumers.contains_key(&enum_.ident) {
            Vec::from_iter(self.enum_consumers.get(&enum_.ident).unwrap().values().cloned())
        } else {
            Vec::new()
        }
    }

    pub fn get_transformed_destructor_signature(
        &self,
        generator_ident: &Ident,
        destructor_ident: &Ident,
    ) -> Signature {
        // If the provided generator_ident is not a trait, find its trait
        let enum_ident = self.get_base_type_name_from_type_name(generator_ident);
        // Get the enum
        self.enum_consumers
            .get(&enum_ident)
            .unwrap()
            .get(&destructor_ident)
            .unwrap()
            .sig
            .clone()
    }

    pub fn get_transformed_consumer_signature(&self, consumer_ident: &Ident) -> Signature {
        self.traits
            .iter()
            .find_map(|trait_| {
                trait_.items.iter().find_map(|item| {
                    if let TraitItem::Method(method) = item {
                        if method.sig.ident == *consumer_ident {
                            return Some(method.sig.clone());
                        }
                    }
                    return None;
                })
            })
            .unwrap()
            .clone()
    }
}

impl<'ast> Visit<'ast> for Gamma {
    fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
        visit_item_enum(self, i);
        self.enums.push(i.clone());
    }

    fn visit_item_trait(&mut self, i: &'ast ItemTrait) {
        visit_item_trait(self, i);
        self.traits.push(i.clone());

        // Filter all the items in the trait and pull out the methods
        let trait_methods = Vec::from_iter(i.items.iter().filter_map(|item| {
            if let TraitItem::Method(impl_item_method) = item {
                return Some(impl_item_method.clone());
            };
            return None;
        }));
        self.destructors.insert(i.ident.clone(), trait_methods);
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        visit_item_struct(self, i);
        self._structs.push(i.clone());
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        visit_item_impl(self, i);

        // Find the trait that is being implemented
        let trait_ident = i.trait_.as_ref().unwrap().1.segments.first().unwrap().ident.clone();

        // Find the struct that the impl is for
        let struct_name: &Ident = if let Type::Path(type_path) = &*i.self_ty {
            &type_path.path.segments.first().unwrap().ident
        } else {
            panic!("Not a path when visiting item_impl");
        };
        let struct_ = self.get_struct_by_name(&struct_name);

        // If the generator doesnt have any generators yet add an empty list
        if !self.generators.contains_key(&trait_ident) {
            self.generators.insert(trait_ident.clone(), Vec::new());
        }

        // Push the struct to the traits generator list
        self.generators
            .get_mut(&trait_ident)
            .unwrap()
            .push((struct_.clone(), i.clone()));
    }

    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        self.functions.push(i.clone());
        // If the first argument of the function is an enum, then it is a consumer so add it to the
        // enum consumers
        if let Some(FnArg::Typed(PatType { ty, .. })) = i.sig.inputs.first() {
            let first_arg_type = ty.get_delta_type().name;
            if self.is_enum(&first_arg_type) {
                self.add_enum_consumer(&self.get_enum(&first_arg_type).unwrap(), &i.sig.ident, i);
            }
        }
    }
}

pub fn generate_gamma(syntax: &syn::File) -> Gamma {
    Gamma::from_file(syntax)
}
