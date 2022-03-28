// Extending the implementation to use THIR/MIR
// https://rustc-dev-guide.rust-lang.org/thir.html
// https://rustc-dev-guide.rust-lang.org/the-parser.html 

use std::collections::HashMap;
use syn::*;
use syn::__private::Span;
use crate::context::*;
use gamma::Gamma;
use errors::*;

#[derive(Debug, Clone)]
pub struct DeltaType {
    pub name: Ident,
    pub is_box: bool,
}

#[derive(Debug, Clone)]
pub struct Delta {
    pub self_ty: Option<Ident>,
    pub types: HashMap<Ident, DeltaType>,
}

pub fn get_struct_attrs(struct_: &ItemStruct) -> Vec<Ident> {
    Vec::from_iter(fields_to_delta_types(&struct_.fields).iter().map(|(field, _)| field.clone()))
}

pub fn get_type_from_box(segment: &PathSegment) -> std::result::Result<Ident, NotABoxType> {
    // If the thing has args
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, ..}) = &segment.arguments {
        let arg = args.first().unwrap();

        // If it is a dyn thing
        if let GenericArgument::Type(Type::TraitObject(TypeTraitObject { bounds, .. })) = arg {
            let bound = bounds.first().unwrap();
            if let TypeParamBound::Trait(TraitBound { path, .. }) = bound {
                return Ok(get_ident_from_path(&path));
            }
        }

        // If not a dyn thing
        if let GenericArgument::Type(Type::Path(type_path)) = arg{
            return Ok(get_ident_from_path(&type_path.path));
        }
    }

    Err(NotABoxType{segment: segment.clone()})
}

/// Check if the provided type is a Box<T>
pub fn is_box(type_: &Type) -> bool {
    return match &type_ {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.first().unwrap();
            segment.ident == "Box"
        }
        _ => panic!("Other types not supported")
    }
}

/// Get the expr inside a Box::new() expression
///
/// If the expr is not a Box::new() expression, resturn failure
pub fn new_box_call_expr(expr: &Expr) -> std::result::Result<Expr, InvalidType> {
    if let Expr::Call(ExprCall{
        func,
        args,
        ..
    }) = &*expr {
        if let Expr::Path(ExprPath{
            path: Path{
                segments,
                ..
            },
            ..
        }) = &**func {
            if segments.first().unwrap().ident == "Box" {
                return Ok(args.first().unwrap().clone())
            }
        }
    }

    return Err(InvalidType{message: "Could not find type".to_string()});
}

pub fn is_dyn_box_generator_return(signature: &Signature, gamma: &Gamma) -> bool {
    if let ReturnType::Type(
        _, type_
    ) = &signature.output {
        if let Type::Path(
            TypePath { path: Path{ segments, .. } , .. }
        ) = &**type_ {
            if let Ok(ident) = get_type_from_box(segments.first().unwrap()) {
                return gamma.is_trait(&ident);
            }
        }
    }
    return false;
}

pub fn get_delta_type_from_type(type_: &Type) -> DeltaType {
    match type_ {
        Type::Path(type_path) => DeltaType{name: get_ident_from_path(&type_path.path), is_box: is_box(&type_)},
        _ => panic!("Other types not supported")
    }
}

pub fn get_ident_from_path(Path { segments, .. }: &Path) -> Ident {
    let segment = segments.first().unwrap();

    if segment.ident == "Box" {
        return get_type_from_box(segment).unwrap();
    }

    return segment.ident.clone();
}

pub fn get_type_from_function_arg(arg: &FnArg, self_type: Option<&Ident>) -> DeltaType{
    if let FnArg::Typed(pat_type) = arg {
        if let Type::Path(type_path) = &*pat_type.ty {
            return DeltaType{name: get_ident_from_path(&type_path.path), is_box: false};
        }
    }
    
    if let FnArg::Receiver(_) = arg {
        if self_type.is_none() {
            panic!("Receiver not supported when self type is None");
        }
        return DeltaType{name: self_type.unwrap().clone(), is_box: false};
    }

    // TODO This will panic for all self types
    panic!("Could not get type from function argument");
}

pub fn get_attribute_ident_from_function_arg(arg: &FnArg) -> Ident {
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

fn fields_to_delta_types(fields: &Fields) -> Vec<(Ident, DeltaType)> {
    match fields {
        Fields::Named(fields_named) => {
            fields_named.named.iter().map(|field|
                (field.ident.clone().unwrap(), get_delta_type_from_type(&field.ty))
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

    pub fn get_type(&self, ident: Ident) -> DeltaType {
        self.types.get(&ident).unwrap_or_else(|| panic!("Type {:?} not in delta. {:?}", ident, self.types)).clone()
    }

    pub fn get_return_type(method_ident: Ident, gamma: &Gamma) {
    }

    pub fn collect_for_struct(&mut self, struct_: &ItemStruct) {
        self.types.extend(
            fields_to_delta_types(&struct_.fields)
        );
    }

    pub fn collect_for_destructor_impl(&mut self, destructor_method_impl: &ImplItemMethod, generator: &ItemStruct) {
        self.self_ty = Some(generator.ident.clone());
        self.collect_for_sig(&destructor_method_impl.sig, Some(&generator.ident));
        self.collect_for_struct(&generator);
    }

    pub fn collect_for_sig(&mut self, signature: &Signature, self_type: Option<&Ident>) {
        let types: HashMap<Ident, DeltaType> = signature.inputs.iter().map(|arg| {
            (get_attribute_ident_from_function_arg(arg), get_type_from_function_arg(arg, self_type))
        }).into_iter().collect();
        self.types.extend(types);
    }

    pub fn collect_for_local(&mut self, local: &Local, gamma: &Gamma) {
        // If the type is specified, use that
        if let Local { pat: Pat::Type(PatType{pat, ty, ..}), .. } = local {
            if let Pat::Ident(PatIdent{ident, ..}) = &**pat {
                self.types.insert(ident.clone(), get_delta_type_from_type(ty));
            }
        }
        
        // Otherwise infer the type
        else if let Local {pat: Pat::Ident(PatIdent{ident, ..}), init, ..} = local {
            if init.is_none() {
                panic!("A let expression must have a body or a type");
            }
            let expr = &init.as_ref().unwrap().1;
            self.types.insert(ident.clone(), self.get_type_of_expr(expr, gamma));
        }
    }

    pub fn get_type_of_expr(&self, expr: &Expr, gamma: &Gamma) -> DeltaType {
        match expr {
            // TODO Match self.thing here so we can do in any order
            Expr::Unary(ExprUnary { expr, .. }) => self.get_type_of_expr(expr, gamma),
            Expr::Path(ExprPath { path, .. }) => self.get_type(get_ident_from_path(path)),
            Expr::Call(ExprCall {func, ..}) if new_box_call_expr(expr).is_ok() => {
                DeltaType{name: self.get_type_of_expr(&new_box_call_expr(expr).unwrap(), gamma).name, is_box: true}
            },
            Expr::Struct(ExprStruct {path, .. }) => DeltaType{
                name: path.segments.first().unwrap().ident.clone(),
                is_box: false
            },
            Expr::MethodCall(ExprMethodCall { receiver, method, ..}) => {
                let receiver_type = self.get_type_of_expr(&receiver, gamma).name;
                // TODO trait does not exist
                let method_sig = gamma.get_destructor_signature(&receiver_type, &method);
                match method_sig.output {
                    ReturnType::Default => panic!("Method {:?} has no return type", method),
                    ReturnType::Type(_, type_) => get_delta_type_from_type(&type_)
                }
            },
            _ => panic!("Unsupported expression: {:?}", expr),
        }
    }
}
