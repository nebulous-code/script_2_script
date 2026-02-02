use crate::scene::{ImageObject, Shape, TextObject};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Shape(Shape),
    Image(ImageObject),
    Text(TextObject),
}
