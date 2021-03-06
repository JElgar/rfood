use std::f64::consts::PI;

trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

struct Circle {
    radius: f64,
}
impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * PI * self.radius
    }
}

struct Square {
    side: f64,
}
impl Shape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }

    fn perimeter(&self) -> f64 {
        4.0 * self.side
    }
}
