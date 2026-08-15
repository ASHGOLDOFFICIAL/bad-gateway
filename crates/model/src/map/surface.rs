/// The base walkable layer of a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface {
    Grass,
    Beach,
    Rock,
}

impl Surface {
    /// Grip coefficient between locomotion and this surface.
    pub fn traction(&self) -> f32 {
        match self {
            Surface::Grass => 0.6,
            Surface::Beach => 0.45,
            Surface::Rock => 0.5,
        }
    }

    /// Rolling resistance coefficient on this surface.
    pub fn rolling_resistance(&self) -> f32 {
        match self {
            Surface::Grass => 0.08,
            Surface::Beach => 0.2,
            Surface::Rock => 0.5,
        }
    }
}
