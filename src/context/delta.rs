// Extending the implementation to use THIR/MIR
// https://rustc-dev-guide.rust-lang.org/thir.html
// https://rustc-dev-guide.rust-lang.org/the-parser.html 

use std::collections::HashMap;
use syn::*;
use syn::__private::Span;
use crate::context::*;
use gamma::Gamma;
use errors::*;

#[derive(Debug, Clone, PartialEq)]
pub enum RefType {
    Box,
    Ref,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EType {
    None,
    DeltaType(DeltaType),
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaType {
    pub name: Ident,
    pub ref_type: RefType,
}
impl DeltaType {
    pub fn is_equaivalent(&self, other: &Self, gamma: &Gamma) -> bool {
        self == other || (self.ref_type == other.ref_type && gamma.is_subtype_of(&self.name, &other.name))
    }
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

pub trait GetOptionalDeltaType {
    fn get_delta_type(&self) -> Option<DeltaType>;
}

impl GetOptionalDeltaType for ReturnType {
    fn get_delta_type(&self) -> Option<DeltaType> {
        match self {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => {
                return Some(ty.get_delta_type());
            }
        }
    }
}

pub trait GetDeltaType {
    fn get_delta_type(&self) -> DeltaType;
}

impl GetDeltaType for Type {
    fn get_delta_type(&self) -> DeltaType {
        match self {
            Type::Path(type_path) => DeltaType{name: get_ident_from_path(&type_path.path), ref_type: self.get_ref_type()},
            Type::Reference(TypeReference { elem, .. }) => {
                elem.get_delta_type()
            }
            _ => panic!("Other types not supported, {:?}", self)
        }
    }
}

impl GetDeltaType for Path {
    fn get_delta_type(&self) -> DeltaType {
        let segment = self.segments.first().unwrap();
    
        if segment.ident == "Box" {
            return DeltaType{name: get_type_from_box(segment).unwrap(), ref_type: RefType::Box};
        }
    
        return DeltaType{name: segment.ident.clone(), ref_type: RefType::None};
    }
}

impl GetDeltaType for FnArg {
    fn get_delta_type(&self) -> DeltaType {
        DeltaType {
            name: match self {
                FnArg::Typed(typed) => typed.ty.get_delta_type().name,
                _ => panic!("Other types not supported, {:?}", self)
            },
            ref_type: self.get_ref_type(),
        }
    }
}

pub trait GetRefType {
    fn get_ref_type(&self) -> RefType;
}

impl GetRefType for Type {
    fn get_ref_type(&self) -> RefType {
        // TODO add in reference types
        return match &self {
            Type::Path(type_path) => {
                let segment = type_path.path.segments.first().unwrap();
                if segment.ident == "Box" {
                    RefType::Box
                } else {
                    RefType::None
                }
            },
            Type::Reference(_) => RefType::Ref,
            _ => RefType::None,
        }
    }
}

impl GetRefType for FnArg {
    fn get_ref_type(&self) -> RefType {
        match self {
            FnArg::Typed(type_pat) => {
                type_pat.ty.get_ref_type()
            }
            FnArg::Receiver(Receiver { reference, .. }) => {
                match reference {
                    Some(_) => RefType::Ref,
                    None => RefType::None,
                }
            },
        }
    }
}

/// Get the expr inside a Box::new() expression
///
/// If the expr is not a Box::new() expression, resturn failure
///
/// # Examples
///
/// Working example:
/// ```
/// use syn::*;
/// use rfood::context::delta::new_box_call_expr;
///
/// let expr = parse_str::<syn::Expr>(r#"Box::new(1)"#).unwrap();
/// let expr = new_box_call_expr(&expr).unwrap();
/// assert!(
///     if let Expr::Lit(ExprLit{lit: Lit::Int(lit_int), ..}) = expr {
///         lit_int.base10_digits() == "1"
///     } else {
///         false
///     }
/// );
/// ```
///
/// Non box should return failure:
/// ```
/// use syn::*;
/// use rfood::context::delta::new_box_call_expr;
///
/// let expr = parse_str::<syn::Expr>(r#"1"#).unwrap();
/// let result = new_box_call_expr(&expr);
/// assert!(matches!(result, std::result::Result::Err(_)));
/// ```
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

pub fn get_ident_from_path(Path { segments, .. }: &Path) -> Ident {
    let segment = segments.first().unwrap();

    if segment.ident == "Box" {
        return get_type_from_box(segment).unwrap();
    }

    return segment.ident.clone();
}

pub fn get_type_from_function_arg(arg: &FnArg, self_type: Option<&Ident>) -> DeltaType{
    // TODO add in reference types
    if let FnArg::Typed(pat_type) = arg {
        match &*pat_type.ty {
            Type::Path(type_path) => return DeltaType{name: get_ident_from_path(&type_path.path), ref_type: RefType::None},
            Type::Reference(TypeReference { elem, .. }) => {
                if let Type::Path(type_path) = &**elem {
                    return DeltaType{name: get_ident_from_path(&type_path.path), ref_type: RefType::None};
                }
            },
            _ => ()
        }
    }
    
    if let FnArg::Receiver(_) = arg {
        if self_type.is_none() {
            panic!("Receiver not supported when self type is None");
        }
        return DeltaType{name: self_type.unwrap().clone(), ref_type: RefType::None};
    }

    // TODO This will panic for all self types
    panic!("Could not get type from function argument, {:?}", arg);
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
                (field.ident.clone().unwrap(), field.ty.get_delta_type())
            ).into_iter().collect()
        },
        _ => panic!("Unanmed structs/enums are not supported")
    }
}

pub fn get_return_type_from_signature(signature: &Signature) -> EType {
    match &signature.output {
        ReturnType::Default => EType::None, 
        ReturnType::Type(_, type_) => {
            EType::DeltaType(type_.get_delta_type())
        }
    }
}

pub fn get_function_call_name(expr_call: &ExprCall) -> Ident{
    match &*expr_call.func {
        Expr::Path(ExprPath{ path, .. }) => get_path_call_name(path),
        _ => panic!("Could not get function name from call")
    }
}

/// Get the name of the function being called from a path
pub fn get_path_call_name(path: &Path) -> Ident {
    path.segments.last().unwrap().ident.clone()
}

impl Delta {
    pub fn new() -> Self {
        return Delta {
            self_ty: None, types: HashMap::new(),
        }
    }

    pub fn get_type(&self, ident: &Ident) -> DeltaType {
        self.types.get(&ident).unwrap_or_else(|| panic!("Type {:?} not in delta. {:?}", ident, self.types)).clone()
    }

    pub fn get_type_of_member(self, member: &Member) -> DeltaType {
        match member {
            Member::Named(member_named) => {
                self.get_type(member_named)
            },
            _ => panic!("Unanmed structs/enums are not supported")
        }
    }

    pub fn collect_for_struct(&mut self, struct_: &ItemStruct) {
        self.types.extend(
            fields_to_delta_types(&struct_.fields)
        );
    }
    
    pub fn collect_for_enum_variant(&mut self, enum_variant: &Variant) {
        self.types.extend(
            fields_to_delta_types(&enum_variant.fields)
        );
    }
    
    pub fn collect_new_for_destructor_impl(&mut self, new_sig: &Signature, generator: &ItemStruct) {
        self.collect_for_sig(&new_sig, None);
        // TODO Catch any overwritting and rename as required
        self.collect_for_struct(&generator);
    }

    pub fn collect_old_for_destructor_impl(&mut self, old_sig: &Signature, generator: &ItemStruct) {
        self.self_ty = Some(generator.ident.clone());
        self.collect_for_sig(old_sig, Some(&generator.ident));
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
                self.types.insert(ident.clone(), ty.get_delta_type());
            }
        }
        
        // Otherwise infer the type
        else if let Local {pat: Pat::Ident(PatIdent{ident, ..}), init, ..} = local {
            if init.is_none() {
                panic!("A let expression must have a body or a type");
            }
            let expr = &init.as_ref().unwrap().1;
            self.types.insert(ident.clone(), self.get_type_of_expr(expr, gamma).unwrap());
        }
    }

    pub fn get_type_of_expr(&self, expr: &Expr, gamma: &Gamma) -> std::result::Result<DeltaType, TypeInferenceFailed> {
        match expr {
            // TODO Match self.thing here so we can do in any order
            Expr::Unary(ExprUnary { expr, .. }) => Ok(self.get_type_of_expr(expr, gamma).unwrap()),
            Expr::Path(ExprPath { path, .. }) => Ok(self.get_type(&get_ident_from_path(path))),
            Expr::Call(ExprCall {..}) if new_box_call_expr(expr).is_ok() => {
                let inner_expr_type_name = self.get_type_of_expr(&new_box_call_expr(expr).unwrap(), gamma);
                if inner_expr_type_name.is_err() {
                    return inner_expr_type_name;
                }
                Ok(DeltaType{name: inner_expr_type_name.unwrap().name, ref_type: RefType::Box})
            },
            // TODO this is wrong if the call is not being transformed, this should be checked for
            // both the call and method call. Might be worth taking in a "transformed" boolean
            Expr::Call(expr_call) => {
                let func_name = get_function_call_name(&expr_call);
                let sig = gamma.get_transformed_consumer_signature(&func_name);
                match get_return_type_from_signature(&sig) {
                    EType::DeltaType(ty) => Ok(ty),
                    _ => panic!("Function {:?} not found", sig)
                }
            }
            Expr::Struct(ExprStruct {path, .. }) => Ok(DeltaType{
                name: path.segments.first().unwrap().ident.clone(),
                ref_type: RefType::None
            }),
            Expr::MethodCall(ExprMethodCall { receiver, method, ..}) => {
                let receiver_type = self.get_type_of_expr(&receiver, gamma);
                if receiver_type.is_err() {
                    return receiver_type;
                }

                // TODO trait does not exist
                let method_sig = gamma.get_transformed_destructor_signature(&receiver_type.unwrap().name, &method);
                match get_return_type_from_signature(&method_sig) {
                    EType::DeltaType(ty) => Ok(ty),
                    _ => panic!("Method {:?} not found", method_sig)
                }
            },
            Expr::Lit(ExprLit{lit, ..}) => {
                match lit {
                    Lit::Int(_) => Ok(DeltaType{name: Ident::new("i32", Span::call_site()), ref_type: RefType::None}),
                    Lit::Float(_) => Ok(DeltaType{name: Ident::new("f32", Span::call_site()), ref_type: RefType::None}),
                    Lit::Bool(_) => Ok(DeltaType{name: Ident::new("bool", Span::call_site()), ref_type: RefType::None}),
                    _ => panic!("Unsupported literal {:?}", lit)
                }
            },
            _ => Err(TypeInferenceFailed{expr: expr.clone()}),
        }
    }
}
