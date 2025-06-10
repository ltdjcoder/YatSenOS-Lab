mod shape;
use shape::{Shape};

fn main() {
    let rectangle = Shape::Rectangle { width: 10.0, height: 20.0 };
    let circle = Shape::Circle { radius: 10.0 };

    println!("Area of rectangle: {}", rectangle.area());
    println!("Area of circle: {}", circle.area());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_area() {
        let rectangle = Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        };
        let circle = Shape::Circle { radius: 10.0 };

        assert_eq!(rectangle.area(), 200.0);
        assert_eq!(circle.area(), 314.1592653589793);
    }
}

