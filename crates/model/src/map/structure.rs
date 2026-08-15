/// An object above surface layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Structure {
    Tree,
    Bush,
    Mountain,
}

impl Structure {
    /// Whether this structure blocks movement.
    pub fn collides(&self) -> bool {
        match self {
            Structure::Tree => true,
            Structure::Bush => false,
            Structure::Mountain => true,
        }
    }
}
