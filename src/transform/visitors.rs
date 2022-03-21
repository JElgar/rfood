use syn::*;
use syn::visit_mut::*;
use syn::punctuated::Punctuated;
use syn::__private::Span;

use crate::context;
use crate::ast;
use context::delta::Delta;
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
}
impl VisitMut for ReplaceFieldCalls {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_expr_mut(self, expr);
        if let syn::Expr::Field(syn::ExprField{
            member: syn::Member::Named(ident),
            base,
            ..
        }) = expr.clone() {
            let member_name = get_method_call_ident(&base);
            if member_name.is_none() {
                return;
            }

            let result_type = self.delta.get_type(member_name.unwrap());
            *expr = syn::Expr::Unary(
                syn::ExprUnary {
                    attrs: Vec::new() as Vec<syn::Attribute>,
                    op: syn::UnOp::Deref(syn::token::Star{spans: [Span::call_site()]}),
                    expr: Box::new(syn::Expr::Path(syn::ExprPath { attrs: Vec::new(), qself: None, path: syn::Path { leading_colon: None, segments: Punctuated::from_iter([syn::PathSegment { ident: ident.clone(), arguments: syn::PathArguments::None}]) } })),
                }
            );
        }
    }
}

pub struct ReplaceMethodCalls {
    pub delta: Delta,
}
impl VisitMut for ReplaceMethodCalls {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_expr_mut(self, expr);
        if let syn::Expr::MethodCall(expr_method_call) = expr.clone() {
            // Extract the type of the expression that the method is being called on
            let expr_type = self.delta.get_type_of_expr(&expr_method_call.receiver);

            // Create function call for method
            // TODO add previous caller to args
            let mut args = Punctuated::from_iter(vec![*expr_method_call.receiver]);
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
            println!("Visiting self expr path {:?}", i);
            *i = create_expr_path_to_ident(&self.enum_name);
        }
    }
}

pub struct ReplaceDynBoxDestructorReturnStatements;
impl VisitMut for ReplaceDynBoxDestructorReturnStatements {
    fn visit_expr_return_mut(&mut self, i: &mut ExprReturn) {
        // If the return statement is a Box::new, remove the box call
        if let Expr::Call(ExprCall{
            func,
            args,
            ..
        }) = &**i.expr.as_ref().unwrap() {
            if let Expr::Path(ExprPath{
                path: Path{
                    segments,
                    ..
                },
                ..
            }) = &**func {
                if segments.first().unwrap().ident == "Box" {
                    // Block
                    // i.expr = Some(Box::new(create_expression_block(
                    //     Vec::from_iter(args.iter().cloned().map(|expr| Stmt::Expr(expr)))))
                    // );
                    i.expr = Some(Box::new(args.first().unwrap().clone()));
                }
            }
        }
    }
}
