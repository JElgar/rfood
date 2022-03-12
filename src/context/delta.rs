// Extending the implementation to use THIR/MIR
// https://rustc-dev-guide.rust-lang.org/thir.html
// https://rustc-dev-guide.rust-lang.org/the-parser.html 

use std::collections::HashMap;
use syn::*;

#[derive(Debug)]
pub struct Delta {
    pub self_ty: Option<Ident>,
    pub types: HashMap<Ident, Ident>,
}

pub fn get_type_from_box(segment: &PathSegment) -> Ident {
    // If the thing has args
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, ..}) = &segment.arguments {
        let arg = args.first().unwrap();

        // If it is a dyn thing
        if let GenericArgument::Type(Type::TraitObject(TypeTraitObject { bounds, .. })) = arg {
            let bound = bounds.first().unwrap();
            if let TypeParamBound::Trait(TraitBound { path, .. }) = bound {
                return get_type_from_path(&path);
            }
        }

        // If not a dyn thing
        if let GenericArgument::Type(Type::Path(type_path)) = arg{
            return get_type_from_path(&type_path.path);
        }
    }

    panic!("Failed to get type from box: {:?}", segment);
}

pub fn get_type_from_path(Path { segments, .. }: &Path) -> Ident {
    let segment = segments.first().unwrap();

    if segment.ident == "Box" {
        return get_type_from_box(segment);
    }

    return segment.ident.clone();
}

pub fn get_type_from_function_arg(arg: &FnArg) -> Ident {
    if let FnArg::Typed(pat_type) = arg {
        if let Type::Path(type_path) = &*pat_type.ty {
            return get_type_from_path(&type_path.path);
        }
    }

    // TODO This will panic for all self types
    panic!("Could not get type from function argument");
}

pub fn get_attribute_ident_fron_function_arg(arg: &FnArg) -> Ident {
    println!("{:?}", arg);
    if let FnArg::Typed(PatType { pat, .. }) = arg {
        if let Pat::Ident(pat_ident) = &**pat {
            return pat_ident.ident.clone();
        }
    }

    // TODO This will panic for all self types
    panic!("Could not get attribute name from function argument");
}

impl Delta {
    pub fn new() -> Self {
        return Delta {
            self_ty: None, types: HashMap::new(),
        }
    }

    pub fn collect_for_destructor_impl(&self, destructor_method_impl: &ImplItemMethod, generator: &ItemStruct) -> Self {
        // TODO append all types from the destructor impl signature
        return Delta {
            self_ty: Some(generator.ident.clone()),
            types: self.collect_for_method_sig(&destructor_method_impl.sig).types,
        }
    }

    pub fn collect_for_method_sig(&self, signature: &Signature) -> Self {
        let types = signature.inputs.iter().filter_map(|arg| {
            if let FnArg::Receiver(_) = arg {
                return None;
            }
            Some((get_attribute_ident_fron_function_arg(arg), get_type_from_function_arg(arg)))
        }).into_iter().collect();

        Delta {
            self_ty: None,
            types,
        }
    }
}
