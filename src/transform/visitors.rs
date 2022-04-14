use syn::*;
use syn::visit_mut::*;
use syn::punctuated::Punctuated;
use syn::__private::Span;

use crate::context;
use crate::ast;
use crate::utils::utils::PopFirst;
use context::delta::{Delta, get_ident_from_path, new_box_call_expr, GetDeltaType, get_function_call_name, clean_type};
use context::gamma::Gamma;
use ast::create::*;

/// Expr is self
fn get_method_call_ident(expr: &Expr) -> Option<Ident> {
    if let syn::Expr::Path(syn::ExprPath{
        path,
        ..
    }) = expr {
        return Some(path.segments.first().unwrap().ident.clone());
    }
    return None;
}


pub struct ReplaceFieldCalls {
    pub delta: Delta,
    pub self_mut_fields: Vec<Ident>,
}
impl VisitMut for ReplaceFieldCalls {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_expr_mut(self, expr);
        if let syn::Expr::Field(syn::ExprField{
            member: syn::Member::Named(ident),
            base,
            ..
        }) = expr.clone() {
            let base_name = get_method_call_ident(&base);
            if base_name.is_none() {
                return;
            }
            let base_name = base_name.unwrap();

            // TODO Should probably use this
            // let result_type = self.delta.get_type(&member_name);

            *expr = syn::Expr::Unary(
                syn::ExprUnary {
                    attrs: Vec::new() as Vec<syn::Attribute>,
                    op: syn::UnOp::Deref(syn::token::Star{spans: [Span::call_site()]}),
                    expr: Box::new(syn::Expr::Path(syn::ExprPath { attrs: Vec::new(), qself: None, path: syn::Path { leading_colon: None, segments: Punctuated::from_iter([syn::PathSegment { ident: ident.clone(), arguments: syn::PathArguments::None}]) } })),
                }
            );
            
            println!("Self mut is: {:?}", self.self_mut_fields);
            if self.self_mut_fields.contains(&ident) {
                println!("Deferencing expr!!!!");
                *expr = clean_type(expr);
            }
        }
    }
}

pub struct ReplaceMethodCalls {
    pub delta: Delta,
    pub gamma: Gamma,
}
impl VisitMut for ReplaceMethodCalls {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_expr_mut(self, expr);
        if let syn::Expr::MethodCall(expr_method_call) = expr.clone() {
            // TODO check the method call is of a transformed type 
            // Extract the type of the expression that the method is being called on

            // Check if the type being transformed is a trait 
            // let expr_type = self.delta.get_type_of_expr(&expr_method_call.receiver, &self.gamma);
            // if !(expr_type.is_ok() && self.gamma.is_trait(&expr_type.unwrap().name)) {
            //     // If not no transformation is needed
            //     
            //     return;
            // }

            // Create function call for method
            let expr_ref = create_reference_of_expr(&*expr_method_call.receiver.clone());
            let mut args = Punctuated::from_iter(vec![expr_ref]);
            args.extend(expr_method_call.args.clone());
            *expr = create_function_call(&expr_method_call.method, args)
        }
    }
}

pub struct ReplaceSelf {
    pub enum_name: Ident,
}
impl VisitMut for ReplaceSelf {
    fn visit_expr_path_mut(&mut self, i: &mut ExprPath) {
        visit_expr_path_mut(self, i);
        if i.path.segments.first().unwrap().ident == "self" {
            *i = create_expr_path_to_ident(&self.enum_name);
        }
    }
}

pub struct ReplaceDynBoxDestructorReturnStatements;
impl VisitMut for ReplaceDynBoxDestructorReturnStatements {
    fn visit_expr_return_mut(&mut self, i: &mut ExprReturn) {
        visit_expr_return_mut(self, i);

        if i.expr.is_none() {
            return;
        }

        // If the return statement is a Box::new, remove the box call
        match new_box_call_expr(&i.clone().expr.unwrap()) {
            Ok(expr) => {
                i.expr = Some(Box::new(expr))
            },
            _ => {
                // TODO find out what I was thinking here with the create_dereference_of_expr
                // i.expr = Some(Box::new(create_dereference_of_expr(i.expr.as_ref().unwrap())))
            }
        }
    }
}

/// When transforming from a consumer to a destructor, we need to add self infront of any literals
/// that come from the enum.
pub struct TransformConsumer {
    pub trait_attributes: Vec<Ident>,
    pub self_arg_name: Ident,
    pub gamma: Gamma,
}
impl VisitMut for TransformConsumer {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        match i {
            Expr::Path(expr_path) | Expr::Unary(ExprUnary { expr: box Expr::Path(expr_path), ..}) => {
                // NOTE See note below
                let var_name = &expr_path.path.segments.first().unwrap().ident.clone();
                // If the path is one of the values in the trait then we need to add self infront
                // of it.
                if self.trait_attributes.contains(&var_name) {
                    *i = create_self_field_call(var_name)
                // Otherwise if the var is the self arg then we need to replace it self
                } else if var_name == &self.self_arg_name {
                    *i = Expr::Path(
                        syn::ExprPath {
                            attrs: Vec::new() as Vec<syn::Attribute>,
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: Punctuated::from_iter(vec![syn::PathSegment {
                                    ident: Ident::new("self", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                }]),
                            }
                        }
                    )
                }
            },
            Expr::Call(expr_call) => {
                let fn_name = get_function_call_name(&expr_call);
                // If the method is not a consumer method then no transformation is needed
                if !self.gamma.is_consumer(&fn_name) {
                    visit_expr_mut(self, i);
                    return;
                }

                // Get the args, removing the first arg as it is self TODO it could not be
                let mut args = expr_call.args.clone();
                let first_arg = args.pop_first().unwrap();

                // TODO check if this is actually self or not
                // let reciever = add_self_to_path(&first_arg);
                // TODO this cleaning is very dubious
                let reciever = clean_type(&first_arg);
                
                // If the function is a consumer, call method on self
                *i = create_method_call(&fn_name, &reciever, &args);
            }
            _ => () 
        }
        visit_expr_mut(self, i)
    }
}

// Replace all generators (structs) with constructors (enums)
pub struct TransformGenerators {
    gamma: Gamma,
    trait_: ItemTrait,
}
impl VisitMut for TransformGenerators {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        // If the expression is a generator
        if let Expr::Struct(expr_struct) = i {
            let expr_ident: Ident = get_ident_from_path(&expr_struct.path);
            // If the struct is a generator of the trait
            if self.gamma.get_generators(&self.trait_.ident).iter().any(|(struct_, _)| struct_.ident == expr_ident) {
                // Add path to the enum
                let enum_path = create_path_for_enum(&self.trait_.ident, &expr_ident);

                expr_struct.path = enum_path;
            }
        }
    }
}

pub struct ReplaceSelfFieldAssignments {
    pub self_fields: Vec<Ident>,
}
impl VisitMut for ReplaceSelfFieldAssignments {
    fn visit_expr_assign_mut(&mut self, i: &mut ExprAssign) {
        visit_expr_assign_mut(self, i);
        match &*i.left {
            Expr::Unary(ExprUnary{
                op: UnOp::Deref(..),
                expr: box Expr::Path(ExprPath {
                    path,
                    ..
                }),
                ..
            }) if self.self_fields.contains(&get_ident_from_path(&path)) => {
                *i.left = create_self_field_call(&get_ident_from_path(&path));
            },
            _ => ()
        }
    }

    fn visit_expr_assign_op_mut(&mut self, i: &mut ExprAssignOp) {
    }
}
