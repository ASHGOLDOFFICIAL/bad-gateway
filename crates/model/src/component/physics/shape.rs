use physics::{
    Area,
    shapes::{Circle, Cylinder, Rect},
};

use crate::component::ComponentResult;

/// An entity's physical body's projection on world map.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    Circle(Circle),
}

impl Projection {
    /// Returns this `Shape`'s [`SurfaceArea`].
    pub fn bounding_box(&self) -> ComponentResult<Rect> {
        match self {
            Projection::Circle(circle) => circle.inscribed_square(),
        }
        .map_err(|err| err.message())
    }
}

/// An entity's physical body.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, derive_more::From)]
pub enum Shape {
    Cylinder(Cylinder),
}

impl Shape {
    /// Returns this `Shape`'s average frontal area.
    #[inline]
    pub fn frontal_area(&self) -> ComponentResult<Area> {
        match self {
            Shape::Cylinder(cylinder) => cylinder.frontal_area(),
        }
        .map_err(|err| err.message())
    }

    /// Returns this `Shape`'s [`SurfaceArea`].
    pub fn surface_area(&self) -> ComponentResult<SurfaceArea> {
        let area = match self {
            Shape::Cylinder(cylinder) => cylinder.surface_area(),
        }
        .map_err(|err| err.message())?;
        Ok(SurfaceArea(area))
    }

    /// Returns this `Shape`'s projection onto world map.
    pub fn projection(&self) -> ComponentResult<Projection> {
        match self {
            Shape::Cylinder(cylinder) => Ok(Projection::Circle(cylinder.base())),
        }
    }

    /// Returns this `Shape`'s drag drag coefficient.
    #[inline]
    pub fn drag_coefficient(&self) -> f32 {
        match self {
            Shape::Cylinder(cylinder) => cylinder.drag_coefficient(),
        }
    }
}

/// Surface area exposed to the environment.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, derive_more::From, derive_more::Into)]
pub struct SurfaceArea(Area);
