use crate::scene::{ImageObject, Shape};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Shape(Shape),
    Image(ImageObject),
}
