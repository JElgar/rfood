trait Shape {
    fn num_sides(&self) -> u8;
    fn perimeter(&self) -> f64;
    fn area(&self) -> f64;
}
struct Circle {
    pub radius: f64,
}
impl Shape for Circle {
    fn num_sides(&self) -> u8 {
        1
    }
    fn perimeter(&self) -> f64 {
        2.0 * PI * self.radius
    }
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}
struct Square {
    pub side: f64,
}
impl Shape for Square {
    fn num_sides(&self) -> u8 {
        4
    }
    fn perimeter(&self) -> f64 {
        4.0 * self.side
    }
    fn area(&self) -> f64 {
        self.side * self.side
    }
}
struct Rectangle {
    pub length: f64,
    pub height: f64,
}
impl Shape for Rectangle {
    fn num_sides(&self) -> u8 {
        4
    }
    fn perimeter(&self) -> f64 {
        2.0 * self.length + 2.0 * self.height
    }
    fn area(&self) -> f64 {
        self.length * self.height
    }
}
const PI: f64 = 3.141592;
