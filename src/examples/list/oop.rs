pub trait List {}

pub struct Empty {}
impl List for Empty {}

pub struct Cons {
    pub x: i32,
    pub xs: Box<dyn List>,
}
impl List for Cons {}

