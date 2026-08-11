use crate::{
    Area, CalculationError, CalculationResult, Length, ValidationError, ValidationResult,
    ops::{CheckedAdd, CheckedMul},
};

/// Empirical drag coefficient table for a cylinder at a given aspect ratio
/// (height / diameter), as found in *Fluid Mechanics* by White, Frank M.
const CYLINDER_DRAG_COEFFICIENT_TABLE: [(f32, f32); 7] = [
    (1.0, 0.64),
    (2.0, 0.68),
    (3.0, 0.72),
    (5.0, 0.74),
    (10.0, 0.82),
    (20.0, 0.91),
    (40.0, 0.98),
];

/// Drag coefficient used for aspect ratios beyond the table's last entry.
/// Uses coefficient for aspect ratio that goes to infinity.
const CYLINDER_DRAG_COEFFICIENT_BEYOND_TABLE: f32 = 1.20;

/// Calculates cylinder drag coefficient by interpolating data from
/// [`CYLINDER_DRAG_COEFFICIENT_TABLE`], clamping to
/// [`CYLINDER_DRAG_COEFFICIENT_BEYOND_TABLE`] beyond its last entry.
fn drag_coefficient(height: f32, diameter: f32) -> f32 {
    debug_assert!(height > 0.0, "height must be positive");
    debug_assert!(diameter > 0.0, "diameter must be positive");

    let aspect_ratio = height / diameter;

    if aspect_ratio <= CYLINDER_DRAG_COEFFICIENT_TABLE[0].0 {
        return CYLINDER_DRAG_COEFFICIENT_TABLE[0].1;
    }

    for window in CYLINDER_DRAG_COEFFICIENT_TABLE.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if aspect_ratio <= x1 {
            let t = (aspect_ratio - x0) / (x1 - x0);
            return y0 + t * (y1 - y0);
        }
    }

    CYLINDER_DRAG_COEFFICIENT_BEYOND_TABLE
}

/// A right circular cylinder, described by its radius and height.
///
/// Carries no position of its own; used to derive aerodynamic properties
/// for a shape positioned elsewhere in the simulation.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cylinder {
    radius: Length,
    height: Length,
}

impl Cylinder {
    /// Creates a `Cylinder` from its `radius` and `height`, which must both be
    /// positive.
    #[inline]
    pub fn new(radius: Length, height: Length) -> ValidationResult<Self> {
        if radius.is_zero() || height.is_zero() {
            Err(ValidationError("cylinder dimensions must be positive"))
        } else {
            Ok(Self { radius, height })
        }
    }

    /// This `Cylinder`'s radius.
    #[inline(always)]
    pub const fn radius(&self) -> Length {
        self.radius
    }

    /// This `Cylinder`'s height.
    #[inline(always)]
    pub const fn height(&self) -> Length {
        self.height
    }

    /// This `Cylinder`'s base diameter.
    #[inline]
    pub fn diameter(&self) -> CalculationResult<Length> {
        self.radius
            .checked_mul(2.0)
            .ok_or(CalculationError::Overflow(
                "diameter overflowed length's valid range",
            ))
    }

    /// Cross-section area of this `Cylinder` facing the direction of travel.
    ///
    /// For right cylinder this area is constant and is independent of
    /// direction.
    #[inline]
    pub fn frontal_area(&self) -> CalculationResult<Area> {
        let diameter = self.diameter()?;
        diameter.area(self.height)
    }

    /// Total surface area of this `Cylinder` exposed to the surrounding
    /// environment.
    #[inline]
    pub fn surface_area(&self) -> CalculationResult<Area> {
        let lateral = self
            .radius
            .area(self.height)?
            .checked_mul(std::f32::consts::TAU)
            .ok_or(CalculationError::Overflow(
                "lateral area overflowed area's valid range",
            ))?;
        let caps = self
            .radius
            .area(self.radius)?
            .checked_mul(std::f32::consts::TAU)
            .ok_or(CalculationError::Overflow(
                "cap area overflowed area's valid range",
            ))?;
        lateral.checked_add(caps).ok_or(CalculationError::Overflow(
            "surface area overflowed area's valid range",
        ))
    }

    /// This `Cylinder`'s drag coefficient, interpolated from its aspect
    /// ratio (height / diameter).
    #[inline]
    pub fn drag_coefficient(&self) -> f32 {
        let diameter = self.radius.as_meters_f32() * 2.0;
        drag_coefficient(self.height.as_meters_f32(), diameter)
    }
}

impl Eq for Cylinder {}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < f32::EPSILON, "{a} !~= {b}");
    }

    fn length(value: f32) -> Length {
        Length::from_meters_f32(value)
    }

    fn cylinder_with_ratio(ratio: f32) -> Cylinder {
        assert!(
            ratio > 0.0,
            "non-positive ratio should not have been passed"
        );
        Cylinder::new(length(0.5), length(ratio)).expect("asserted above")
    }

    #[test]
    fn new_rejects_zero_dimensions() {
        assert!(Cylinder::new(Length::ZERO, length(1.0)).is_err());
        assert!(Cylinder::new(length(1.0), Length::ZERO).is_err());
    }

    #[test]
    fn base_diameter() {
        let cylinder = Cylinder::new(length(2.0), length(3.0)).unwrap();
        assert_eq!(cylinder.diameter().unwrap().as_meters_f32(), 4.0);
    }

    #[test]
    fn frontal_area() {
        let cylinder = Cylinder::new(length(3.0), length(5.0)).unwrap();
        assert_eq!(
            cylinder.frontal_area().unwrap().as_square_meters_f32(),
            30.0
        );
    }

    #[test]
    fn surface_area() {
        let cylinder = Cylinder::new(length(2.0), length(3.0)).unwrap();
        // lateral = 2*pi*r*h = 2*pi*2*3 = 12*pi; caps = 2*pi*r^2 = 8*pi.
        let expected = 12.0 * std::f32::consts::PI + 8.0 * std::f32::consts::PI;
        approx_eq(
            cylinder.surface_area().unwrap().as_square_meters_f32(),
            expected,
        );
    }

    #[test]
    fn drag_coefficient_clamps_at_or_below_first_entry() {
        let (first_ratio, first_coef) = CYLINDER_DRAG_COEFFICIENT_TABLE
            .first()
            .copied()
            .expect("table is not empty");

        let cylinder = cylinder_with_ratio(first_ratio);
        approx_eq(cylinder.drag_coefficient(), first_coef);

        let semiflat = cylinder_with_ratio(first_ratio / 2.0);
        approx_eq(semiflat.drag_coefficient(), first_coef);
    }

    #[test]
    fn drag_coefficient_interpolates_between_entries() {
        let (first_ratio, first_coef) = CYLINDER_DRAG_COEFFICIENT_TABLE
            .first()
            .copied()
            .expect("table is not empty");

        let (second_ratio, second_coef) = CYLINDER_DRAG_COEFFICIENT_TABLE
            .get(1)
            .copied()
            .expect("table has at least two elements");

        let midpoint_ratio = 0.5 * first_ratio + 0.5 * second_ratio;
        let midpoint_coef = 0.5 * first_coef + 0.5 * second_coef;

        let cylinder = cylinder_with_ratio(midpoint_ratio);
        approx_eq(cylinder.drag_coefficient(), midpoint_coef);
    }

    #[test]
    fn drag_coefficient_matches_last_table_entry_exactly() {
        let (last_ratio, last_coef) = CYLINDER_DRAG_COEFFICIENT_TABLE
            .last()
            .copied()
            .expect("table is not empty");

        let cylinder = cylinder_with_ratio(last_ratio);
        approx_eq(cylinder.drag_coefficient(), last_coef);
    }

    #[test]
    fn drag_coefficient_clamps_beyond_last_table_entry() {
        let (last_ratio, _) = CYLINDER_DRAG_COEFFICIENT_TABLE
            .last()
            .copied()
            .expect("table is not empty");

        let cylinder = cylinder_with_ratio(last_ratio + 1.0);
        approx_eq(
            cylinder.drag_coefficient(),
            CYLINDER_DRAG_COEFFICIENT_BEYOND_TABLE,
        );
    }
}
