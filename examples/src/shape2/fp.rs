pub enum Shape {
    Circle,
    Triangle,
}
pub fn side_count(shape: &Shape) -> i32 {
    match &shape {
        Shape::Circle {} => 1,
        Shape::Triangle {} => 3,
    }
}
pub fn internal_angle(shape: &Shape) -> i32 {
    match &shape {
        Shape::Circle {} => 0,
        _ => 180 * (side_count(&shape) - 2),
    }
}
