use std::f64::consts::PI;

enum Shape {
    Circle { radius: f64 },
    Square { side: f64 },
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius } => 
            PI * radius * radius,
        Shape::Square { side } => 
            side * side,
    }
}

fn perimeter(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius } => 
            2.0 * PI * radius,
        Shape::Square { side } => 
            4.0 * side,
    }
}
