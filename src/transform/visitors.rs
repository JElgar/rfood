use syn::*;
use syn::visit_mut::*;
use syn::punctuated::Punctuated;
use syn::__private::Span;

use crate::context;
use crate::ast;
use context::delta::Delta;
use ast::create::create_function_call;

/// Expr is self
fn get_method_call_ident(expr: &Expr) -> Option<Ident> {
    println!("Checking {:?} is self", expr);
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
        println!("Visiting expr, {:?}", expr);
        visit_expr_mut(self, expr);
        if let syn::Expr::Field(syn::ExprField{
            member: syn::Member::Named(ident),
            base,
            ..
        }) = expr.clone() {
            println!("Found field expr");
            let member_name = get_method_call_ident(&base);
            println!("Member name is {:?}", member_name);
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
        println!("Visiting expr, {:?}", expr);
        visit_expr_mut(self, expr);
        if let syn::Expr::MethodCall(expr_method_call) = expr.clone() {
            println!("Found method call expr {:?}", expr_method_call);

            // Extract the type of the expression that the method is being called on
            let expr_type = self.delta.get_type_of_expr(&expr_method_call.receiver);
            println!("Expr type is {:?}", expr_type);

            // Create function call for method
            // TODO add previous caller to args
            let mut args = expr_method_call.args.clone();
            args.push(*expr_method_call.receiver);
            *expr = create_function_call(&expr_method_call.method, args)
        }
    }
}
