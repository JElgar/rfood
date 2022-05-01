// Extending the implementation to use THIR/MIR
// https://rustc-dev-guide.rust-lang.org/thir.html
// https://rustc-dev-guide.rust-lang.org/the-parser.html 

use std::collections::HashMap;
use syn::*;
use syn::__private::Span;
use crate::context::*;
use crate::ast::create::{remove_deference_of_expr, remove_reference_of_expr};
use gamma::Gamma;
use errors::*;

#[derive(Debug, Clone, PartialEq)]
pub enum RefType {
    Box(Box<RefType>),
    Ref(Box<RefType>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EType {
    None,
    DeltaType(DeltaType),
    RefType(RefType),
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaType {
    pub name: Ident,
    pub ref_type: RefType,
}
impl DeltaType {
    pub fn new(name: &str, ref_type: RefType) -> Self {
        DeltaType {
            name: Ident::new(name, Span::call_site()),
            ref_type
        }
    }
    pub fn is_equaivalent(&self, other: &Self, gamma: &Gamma) -> bool {
        self == other || (self.ref_type == other.ref_type && gamma.is_subtype_of(&self.name, &other.name))
    }

    pub fn replace_self(&self, self_type: Option<Ident>) -> DeltaType {
        if self.name == "Self" {
            return DeltaType {
                name: self_type.unwrap_or_else(|| panic!("Self type not provided")),
                ..self.clone()
            }
        }
        return self.clone();
    }
}

#[derive(Debug, Clone)]
pub struct Delta {
    pub self_ty: Option<Ident>,
    pub types: HashMap<Ident, DeltaType>,
}

pub fn get_struct_attrs(struct_: &ItemStruct) -> Vec<Ident> {
    Vec::from_iter(fields_to_delta_types(&struct_.fields, false).iter().map(|(field, _)| field.clone()))
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

pub trait GetOptionalDeltaTypeFn {
    fn get_delta_type(&self, self_type: Option<Ident>) -> Option<DeltaType>;
}

impl GetOptionalDeltaTypeFn for ReturnType {
    fn get_delta_type(&self, self_type: Option<Ident>) -> Option<DeltaType> {
        match self {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => {
                return Some(ty.get_delta_type().replace_self(self_type));
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
            return DeltaType{name: get_type_from_box(segment).unwrap(), ref_type: RefType::Box(Box::new(RefType::None))};
        }
    
        return DeltaType{name: segment.ident.clone(), ref_type: RefType::None};
    }
}

pub trait GetDeltaTypeFn {
    fn get_delta_type(&self, self_type: Option<Ident>) -> DeltaType;
}

impl GetDeltaTypeFn for FnArg {
    fn get_delta_type(&self, self_type: Option<Ident>) -> DeltaType {
        DeltaType {
            name: match self {
                FnArg::Typed(typed) => typed.ty.get_delta_type().name,
                FnArg::Receiver(_) => self_type.clone().unwrap(),
            },
            ref_type: self.get_ref_type(),
        }.replace_self(self_type)
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
                    RefType::Box(Box::new(RefType::None))
                } else {
                    RefType::None
                }
            },
            Type::Reference(_) => RefType::Ref(Box::new(RefType::None)),
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
                    Some(_) => RefType::Ref(Box::new(RefType::None)),
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
    let delta_type = match &arg {
        FnArg::Typed(PatType{ty: box Type::Path(type_path), ..}) => {
            type_path.path.get_delta_type()
        }
        FnArg::Typed(PatType{ty: box Type::Reference(TypeReference{ elem: box Type::Path(type_path), .. }), ..}) => {
            type_path.path.get_delta_type()
        }
        FnArg::Receiver(_) => {
            if self_type.is_none() {
                panic!("Receiver not supported when self type is None");
            }
            DeltaType{name: self_type.unwrap().clone(), ref_type: RefType::None}
        }
        _ => panic!("Could not get type from function argument, {:?}", arg),
    };

    delta_type.replace_self(self_type.cloned())
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

fn fields_to_delta_types(fields: &Fields, is_ref: bool) -> Vec<(Ident, DeltaType)> {
    match fields {
        Fields::Named(fields_named) => {
            fields_named.named.iter().map(|field| {
                let dt = field.ty.get_delta_type();
                (field.ident.clone().unwrap(), DeltaType{
                    name: dt.name,
                    ref_type: if is_ref && dt.ref_type == RefType::None {
                        RefType::Ref(Box::new(dt.ref_type))
                    } else {
                        dt.ref_type 
                    }})
            }).into_iter().collect()
        },
        Fields::Unit => vec![],
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
    get_expr_call_name(&*expr_call.func)
}

pub fn get_expr_call_name(expr: &Expr) -> Ident {
    match &expr {
        Expr::Path(ExprPath{ path, .. }) => get_path_call_name(path),
        _ => panic!("Could not get function name from call")
    }
}

/// Remove any reference/dereference from expr
pub fn clean_type(expr: &Expr) -> Expr {
    remove_reference_of_expr(&remove_deference_of_expr(&expr))
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

    pub fn collect_for_struct(&mut self, struct_: &ItemStruct, struct_ref_type: RefType) {
        let mut field_type = fields_to_delta_types(&struct_.fields, false);
        if matches!(struct_ref_type, RefType::Ref(_)) {
            field_type.iter_mut().for_each(|(_, delta_type)| {
                delta_type.ref_type = RefType::Ref(Box::new(RefType::None));
            });
        }
        self.types.extend(
            field_type
        );
    }
    
    pub fn collect_for_enum_variant(&mut self, enum_variant: &Variant, is_ref: bool) {
        self.types.extend(
            fields_to_delta_types(&enum_variant.fields, is_ref)
        );
    }
    
    pub fn collect_new_for_destructor_impl(&mut self, new_sig: &Signature, generator: &ItemStruct) {
        self.collect_for_sig(&new_sig, None);
        // TODO Catch any overwritting and rename as required
        self.collect_for_struct(&generator, RefType::Ref(Box::new(RefType::None)));
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

    /// Collect delta info from 
    pub fn collect_for_arm(&mut self, arm: &Arm, gamma: &Gamma) {
        if let Pat::Struct(PatStruct{
            path,
            fields,
            ..
        }) = &arm.pat {
            // Get the type of the thing being matched
            let enum_name = get_path_call_name(&path);
            let variant = gamma.get_constructor(&enum_name);

            // Get the type of the fields
            self.collect_for_enum_variant(&variant.unwrap(), true);
        }
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
            // TODO add handling reference
            Expr::Unary(ExprUnary { expr, op, .. }) => {
                let type_ = self.get_type_of_expr(expr, gamma);
                match (op, &type_) {
                    (UnOp::Deref(_), Ok(DeltaType{
                        name,
                        ref_type: RefType::Box(inner_ref_type) | RefType::Ref(inner_ref_type),
                    })) => Ok(DeltaType{name: name.clone(), ref_type: *inner_ref_type.clone()}),
                    _ => type_
                }
            },
            Expr::Reference(ExprReference{ expr, .. }) => {
                let inner_expr_type = self.get_type_of_expr(expr, gamma);
                if inner_expr_type.is_err() {
                    return inner_expr_type;
                }

                let inner_expr_type = inner_expr_type.unwrap();
                Ok(DeltaType{
                    name: inner_expr_type.clone().name,
                    ref_type: RefType::Ref(Box::new(inner_expr_type.clone().ref_type))
                })
            },
            Expr::Path(ExprPath { path, .. }) => {
                Ok(self.get_type(&get_ident_from_path(path)))
            },
            Expr::Call(ExprCall {..}) if new_box_call_expr(expr).is_ok() => {
                let inner_expr_type = self.get_type_of_expr(&new_box_call_expr(expr).unwrap(), gamma);
                if inner_expr_type.is_err() {
                    return inner_expr_type;
                }
                let inner_expr_type = inner_expr_type.unwrap();
                Ok(DeltaType{name: inner_expr_type.name, ref_type: RefType::Box(Box::new(inner_expr_type.ref_type))})
            },
            // TODO this is wrong if the call is not being transformed, this should be checked for
            // both the call and method call. Might be worth taking in a "transformed" boolean
            Expr::Call(expr_call) => {
                let func_name = get_function_call_name(&expr_call);
                // NOTE I just changed this I think itll cause chaos
                // let sig = gamma.get_transformed_consumer_signature(&func_name);
                let sig = gamma.get_signature(&func_name).unwrap();

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
                let receiver_type = receiver_type.unwrap();

                if method == "clone" {
                    return Ok(receiver_type);
                }

                let method_sig = gamma.get_destructor_signature(&receiver_type.name, &method).unwrap();

                match get_return_type_from_signature(&method_sig) {
                    EType::DeltaType(ty) => Ok(ty),
                    _ => panic!("Method {:?} not found", method_sig)
                }
            },
            Expr::Field(ExprField {base, member, .. }) => {
                let base_type = self.get_type_of_expr(&base, gamma);
                if base_type.is_err() {
                    return base_type;
                }
                match member {
                    Member::Named(member_ident) => Ok(gamma.get_type_of_field(&base_type.unwrap().name, &member_ident)),
                    _ => panic!("Unanmed structs/enums are not supported")
                }
            },
            Expr::Lit(ExprLit{lit, ..}) => {
                match lit {
                    Lit::Int(_) => Ok(DeltaType{name: Ident::new("i32", Span::call_site()), ref_type: RefType::None}),
                    Lit::Float(_) => Ok(DeltaType{name: Ident::new("f32", Span::call_site()), ref_type: RefType::None}),
                    Lit::Bool(_) => Ok(DeltaType{name: Ident::new("bool", Span::call_site()), ref_type: RefType::None}),
                    Lit::Str(_) => Ok(DeltaType{name: Ident::new("str", Span::call_site()), ref_type: RefType::None}),
                    _ => panic!("Unsupported literal {:?}", lit)
                }
            },
            Expr::Binary(ExprBinary { left, right, op, .. }) => {
                match op {
                    BinOp::Eq(_) | BinOp::Ne(_) | BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_)  => Ok(DeltaType{name: Ident::new("bool", Span::call_site()), ref_type: RefType::None}),
                    BinOp::Add(_) | BinOp::Div(_) | BinOp::Sub(_) | BinOp::Mul(_) | BinOp::Rem(_) => self.get_type_of_expr(left, gamma),
                    BinOp::And(_) | BinOp::Or(_) => Ok(DeltaType{name: Ident::new("bool", Span::call_site()), ref_type: RefType::None}),
                    _ => panic!("Unsupported op {:?}", op)
                }
            },
            Expr::Paren(ExprParen { expr, .. }) => self.get_type_of_expr(expr, gamma),
            _ => Err(TypeInferenceFailed{expr: expr.clone()}),
        }
    }
}
