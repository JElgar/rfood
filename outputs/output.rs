enum Shape {
    Circle { radius: f64 },
    Square { side: f64 },
}
fn area(shape: &Shape) -> f64 {
    match &shape {
        Shape::Circle { radius } => 3.14 * *radius * *radius,
        Shape::Square { side } => *side * side,
    }
}
fn perimeter(shape: &Shape) -> f64 {
    match &shape {
        Shape::Circle { radius } => 2.0 * 3.14 * *radius,
        Shape::Square { side } => 4.0 * *side,
    }
}
