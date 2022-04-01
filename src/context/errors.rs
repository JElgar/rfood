use syn::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct NotABoxType {
    pub segment: PathSegment,
}
impl fmt::Display for NotABoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to get type from box: {:?}", self.segment)
    }
}

#[derive(Debug, Clone)]
pub struct NotFound {
    pub item_name: String,
    pub type_name: String,
}
impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to find {:?} of type {:?}", self.item_name, self.type_name)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidType {
    pub message: String,
}
impl fmt::Display for InvalidType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct TypeInferenceFailed {
    pub expr: Expr,
}
impl fmt::Display for TypeInferenceFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Type inference failed for type {:?}", self.expr)
    }
}
