use crate::model::shape::ShapeType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    Pointer,
    Text,
    Shape(ShapeType),
    Image,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Pointer
    }
}
