use std::f64::consts::PI;


pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    // Triangle { base: f64, height: f64 },
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            // Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
}
