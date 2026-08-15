//! Formulas that combine multiple physics quantities into a derived result.

use crate::{
    Angle, Area, CalculationError, CalculationResult, Density, Force, ForceMagnitude, Mass, Power,
    Speed, Temperature, Velocity, ops,
};

/// Drag opposing `velocity`.
///
/// `F(drag) = 0.5 * Cd * rho * A * v * |v|`, where
/// - `F(drag)` - drag force, dimension MLT⁻².
/// - `Cd` - drag coefficient, dimensionless.
/// - `rho` - air density, dimension ML⁻³.
/// - `A` - cross-section facing the direction of travel, dimension L².
/// - `v` - velocity, dimension LT⁻¹.
pub fn drag(
    drag_coefficient: f32,
    frontal_area: Area,
    air_density: Density,
    velocity: Velocity,
) -> CalculationResult<Force> {
    if !drag_coefficient.is_finite() {
        return Err(CalculationError::InvalidArgument(
            "drag coefficient must be finite",
        ));
    }
    let speed = velocity.speed()?.as_meters_per_second_f32();
    let magnitude = 0.5
        * drag_coefficient
        * air_density.as_kilograms_per_cubic_meter_f32()
        * frontal_area.as_square_meters_f32()
        * speed
        * speed;
    let drag_vec = -velocity.as_meters_per_second_vec2().normalize_or_zero() * magnitude;
    Force::try_from_newtons_vec2(drag_vec).map_err(CalculationError::from)
}

/// Terrain traction that allows movement.
///
/// `F(traction) = mu * g * m`, where
/// - `F(traction)` - traction force, dimension MLT⁻².
/// - `mu` - traction coefficient, dimensionless.
/// - `g` - gravity, dimension LT⁻².
/// - `m` - mass, dimension M.
pub const fn traction(
    traction_coefficient: f32,
    gravity: f32,
    mass: Mass,
) -> CalculationResult<ForceMagnitude> {
    if !traction_coefficient.is_finite() || !gravity.is_finite() {
        return Err(CalculationError::InvalidArgument(
            "gravity and coefficient must be finite",
        ));
    }
    match ForceMagnitude::try_from_newtons_f32(
        traction_coefficient * gravity * mass.as_kilograms_f32(),
    ) {
        Ok(traction) => Ok(traction),
        Err(_) => Err(CalculationError::Overflow(
            "traction must be finite and non-negative",
        )),
    }
}

/// Rolling resistance opposing `velocity`.
///
/// `F(rolling) = Crr * g * m`, where
/// - `F(rolling)` - rolling resistance force, dimension MLT⁻².
/// - `Crr` - rolling resistance coefficient, dimensionless.
/// - `g` - gravity, dimension LT⁻².
/// - `m` - mass, dimension M.
pub fn rolling_resistance(
    rolling_resistance_coefficient: f32,
    gravity: f32,
    mass: Mass,
    velocity: Velocity,
) -> CalculationResult<Force> {
    if !rolling_resistance_coefficient.is_finite() || !gravity.is_finite() {
        return Err(CalculationError::InvalidArgument(
            "gravity and coefficient must be finite",
        ));
    }
    let magnitude = rolling_resistance_coefficient * gravity * mass.as_kilograms_f32();
    let velocity = velocity.as_meters_per_second_vec2();
    let rolling_resistance_vec = -velocity.normalize_or_zero() * magnitude;
    Force::try_from_newtons_vec2(rolling_resistance_vec).map_err(CalculationError::from)
}

/// [`Force`] an engine delivers along `direction`.
///
/// `F(engine) = P / v`, where
/// - `F(engine)` - engine force, dimension MLT⁻².
/// - `P` - engine power, dimension ML²T⁻³.
/// - `v` - object's current speed, dimension LT⁻¹.
///
/// Errors if `speed` is zero, or so close to zero that the derived force
/// would overflow `Force`'s valid range.
pub fn engine_force(power: Power, speed: Speed, direction: Angle) -> CalculationResult<Force> {
    let speed = speed.as_meters_per_second_f32();
    let magnitude = power.as_watts_f32() / speed;
    Force::try_from_newtons_vec2(direction.as_vec2_f32() * magnitude)
        .map_err(CalculationError::from)
}

/// The heat exchanged between a surface and its environment.
///
/// `q = h * (T(env) - T) * A`, where
/// - `q` - heat flow into the surface, dimension ML²T⁻³.
/// - `h` - heat transfer coefficient, dimension MT⁻³Θ⁻¹.
/// - `T` - the surface's temperature, dimension Θ.
/// - `T(env)` - the environment's temperature, dimension Θ.
/// - `A` - the surface area exposed to the environment, dimension L².
///
/// Derived from Newton's law of cooling.
pub fn newtonian_cooling(
    surface: Temperature,
    environment: Temperature,
    heat_transfer_coefficient: f32,
    surface_area: Area,
) -> CalculationResult<ops::Delta<Power>> {
    if !heat_transfer_coefficient.is_finite() {
        return Err(CalculationError::InvalidArgument(
            "coefficient must be finite",
        ));
    }
    ops::Delta::between(environment, surface)
        .map(|delta| {
            let heat_flux = heat_transfer_coefficient * delta.as_kelvins_f32();
            let watts = heat_flux * surface_area.as_square_meters_f32();
            Power::try_from_watts_f32(watts).map_err(CalculationError::from)
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_opposes_velocity() {
        let velocity = Velocity::from_meters_per_second_vec2(glam::Vec2::new(10.0, 0.0));
        let air_density = Density::from_kilograms_per_cubic_meter_f32(1.2);
        let frontal_area = Area::from_square_meters_f32(2.0);
        let force = drag(1.0, frontal_area, air_density, velocity);
        assert_eq!(
            force.unwrap().as_newtons_vec2().normalize(),
            -velocity.as_meters_per_second_vec2().normalize()
        );
    }

    #[test]
    fn drag_is_zero_at_zero_velocity() {
        let velocity = Velocity::ZERO;
        let air_density = Density::from_kilograms_per_cubic_meter_f32(1.2);
        let frontal_area = Area::from_square_meters_f32(2.0);
        let force = drag(1.0, frontal_area, air_density, velocity);
        assert!(force.unwrap().is_zero());
    }

    #[test]
    fn rolling_resistance_opposes_velocity_independent_of_speed() {
        let mass = Mass::from_kilograms_f32(1_000.0);
        let velocity = Velocity::from_meters_per_second_vec2(glam::Vec2::new(1.0, 0.0));
        let force = rolling_resistance(0.02, 9.8, mass, velocity);
        assert_eq!(
            force.unwrap().as_newtons_vec2().normalize(),
            -velocity.as_meters_per_second_vec2().normalize()
        );
    }

    #[test]
    fn rolling_resistance_is_independent_of_speed() {
        let mass = Mass::from_kilograms_f32(1_000.0);
        let slow = Velocity::from_meters_per_second_vec2(glam::Vec2::new(1.0, 0.0));
        let fast = Velocity::from_meters_per_second_vec2(glam::Vec2::new(100.0, 0.0));

        let slow_force = rolling_resistance(0.02, 9.8, mass, slow).unwrap();
        let fast_force = rolling_resistance(0.02, 9.8, mass, fast).unwrap();
        assert_eq!(slow_force, fast_force);
    }

    #[test]
    fn rolling_resistance_is_zero_at_zero_velocity() {
        let mass = Mass::from_kilograms_f32(1_000.0);
        let velocity = Velocity::ZERO;
        let force = rolling_resistance(0.02, 9.8, mass, velocity);
        assert!(force.unwrap().is_zero());
    }

    #[test]
    fn engine_force_rejects_near_zero_speed() {
        let power = Power::from_watts_f32(1_000.0);
        let speed = Velocity::ZERO.speed().unwrap();
        let direction = Angle::from_radians_f32(0.0);
        assert!(engine_force(power, speed, direction).is_err());
    }

    #[test]
    fn newtonian_cooling_flows_into_cooler_surface() {
        let surface = Temperature::from_kelvins_f32(280.0);
        let ambient = Temperature::from_kelvins_f32(300.0);
        let area = Area::from_square_meters_f32(2.0);
        let flow = newtonian_cooling(surface, ambient, 5.0, area).unwrap();
        assert_eq!(flow, ops::Delta::Positive(Power::from_watts_f32(200.0)));
    }

    #[test]
    fn newtonian_cooling_flows_out_of_warmer_surface() {
        let surface = Temperature::from_kelvins_f32(300.0);
        let ambient = Temperature::from_kelvins_f32(280.0);
        let area = Area::from_square_meters_f32(2.0);
        let flow = newtonian_cooling(surface, ambient, 5.0, area).unwrap();
        assert_eq!(flow, ops::Delta::Negative(Power::from_watts_f32(200.0)));
    }

    #[test]
    fn newtonian_cooling_is_none_at_equilibrium() {
        let temperature = Temperature::from_kelvins_f32(290.0);
        let area = Area::from_square_meters_f32(2.0);
        assert_eq!(
            newtonian_cooling(temperature, temperature, 5.0, area).unwrap(),
            ops::Delta::Zero
        );
    }
}
