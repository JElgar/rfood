pub trait List {}

pub struct Empty {}
impl List for Empty {}

pub struct Cons {
    x: i32,
    xs: Box<List>,
}
impl List for Cons {}

