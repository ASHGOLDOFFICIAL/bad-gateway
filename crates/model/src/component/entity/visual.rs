/// Texture type for this entity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureId {
    Player,
    Bullets,
}

/// This entity is visible by user and should be drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Visual {
    pub texture_id: TextureId,
}

impl Visual {
    /// Creates new `Visual` marker with given texture.
    #[inline(always)]
    pub const fn new(texture_id: TextureId) -> Self {
        Self { texture_id }
    }
}
