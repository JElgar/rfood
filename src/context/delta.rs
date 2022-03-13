// Extending the implementation to use THIR/MIR
// https://rustc-dev-guide.rust-lang.org/thir.html
// https://rustc-dev-guide.rust-lang.org/the-parser.html 

use std::collections::HashMap;
use syn::*;
use syn::__private::Span;

#[derive(Debug, Clone)]
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
                return get_ident_from_path(&path);
            }
        }

        // If not a dyn thing
        if let GenericArgument::Type(Type::Path(type_path)) = arg{
            return get_ident_from_path(&type_path.path);
        }
    }

    panic!("Failed to get type from box: {:?}", segment);
}

fn get_type_ident_from_type(type_: &Type) -> Ident {
    match type_ {
        Type::Path(type_path) => get_ident_from_path(&type_path.path),
        _ => panic!("Other types not supported")
    }
}

pub fn get_ident_from_path(Path { segments, .. }: &Path) -> Ident {
    let segment = segments.first().unwrap();

    if segment.ident == "Box" {
        return get_type_from_box(segment);
    }

    return segment.ident.clone();
}

pub fn get_type_from_function_arg(arg: &FnArg, self_type: &Ident) -> Ident {
    if let FnArg::Typed(pat_type) = arg {
        if let Type::Path(type_path) = &*pat_type.ty {
            return get_ident_from_path(&type_path.path);
        }
    }
    
    if let FnArg::Receiver(_) = arg {
        return self_type.clone();
    }

    // TODO This will panic for all self types
    panic!("Could not get type from function argument");
}

pub fn get_attribute_ident_from_function_arg(arg: &FnArg) -> Ident {
    println!("{:?}", arg);
    if let FnArg::Typed(PatType { pat, .. }) = arg {
        if let Pat::Ident(pat_ident) = &**pat {
            return pat_ident.ident.clone();
        }
    }

    if let FnArg::Receiver(_) = arg {
        return Ident::new("self", Span::call_site());
    }

    // TODO This will panic for all self types
    panic!("Could not get attribute name from function argument");
}

fn fields_to_delta_types(fields: &Fields) -> HashMap<Ident, Ident> {
    match fields {
        Fields::Named(fields_named) => {
            fields_named.named.iter().map(|field|
                (field.ident.clone().unwrap(), get_type_ident_from_type(&field.ty))
            ).into_iter().collect()
        },
        _ => panic!("Unanmed structs are not supported")
    }
}

impl Delta {
    pub fn new() -> Self {
        return Delta {
            self_ty: None, types: HashMap::new(),
        }
    }

    pub fn get_type(&self, ident: Ident) -> Ident {
        self.types.get(&ident).unwrap_or_else(|| panic!("Type {:?} not in delta. {:?}", ident, self.types)).clone()
    }

    pub fn collect_for_struct(&mut self, struct_: &ItemStruct) {
        self.types.extend(
            fields_to_delta_types(&struct_.fields)
        );
    }

    pub fn collect_for_destructor_impl(&mut self, destructor_method_impl: &ImplItemMethod, generator: &ItemStruct) {
        self.self_ty = Some(generator.ident.clone());
        self.collect_for_method_sig(&destructor_method_impl.sig, &generator.ident);
        self.collect_for_struct(&generator);
    }

    pub fn collect_for_method_sig(&mut self, signature: &Signature, self_type: &Ident) {
        let types: HashMap<Ident, Ident> = signature.inputs.iter().map(|arg| {
            (get_attribute_ident_from_function_arg(arg), get_type_from_function_arg(arg, self_type))
        }).into_iter().collect();

        println!("For sig: {:?} \nDelta types are: {:?}", signature, types);
        self.types.extend(types);
    }

    pub fn get_type_of_expr(&self, expr: &Expr) -> Ident {
        match expr {
            // TODO Match self.thing here so we can do in any order
            Expr::Unary(ExprUnary { expr, .. }) => self.get_type_of_expr(expr),
            Expr::Path(ExprPath { path, .. }) => self.get_type(get_ident_from_path(path)),
            _ => panic!("Unsupported expression: {:?}", expr),
        }
    }
}