struct Shape {
    name: String,
}

trait ShapeFactory {
    fn create_shape(&self) -> Shape;
}

struct CircleFactory {}

impl ShapeFactory for CircleFactory {
    fn create_shape(&self) -> Shape {
        Shape {
            name: "Circle".to_string(),
        }
    }
}

struct SquareFactory {}

impl ShapeFactory for SquareFactory {
    fn create_shape(&self) -> Shape {
        Shape {
            name: "Square".to_string(),
        }
    }
}

fn main() {
    let circle_factory = CircleFactory {};
    let circle = circle_factory.create_shape();
    assert_eq!(circle.name, "Circle");

    let square_factory = SquareFactory {};
    let square = square_factory.create_shape();
    assert_eq!(square.name, "Square");
}
