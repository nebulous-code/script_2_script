use crate::scene::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { radius: f32, color: Color },
    Rect { width: f32, height: f32, color: Color },
}
