pub trait Shape {
    fn side_count(&self) -> i32;
    fn internal_angle(&self) -> i32 {
        180 * (self.side_count() - 2)
    }
}

pub struct Circle;
impl Shape for Circle {
    fn side_count(&self) -> i32 {
        1
    }
    fn internal_angle(&self) -> i32 {
        0
    }
}

pub struct Triangle;
impl Shape for Triangle {
    fn side_count(&self) -> i32 {
        3
    }
}

