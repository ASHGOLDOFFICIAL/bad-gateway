use model::map::{Cell, Liquid, LiquidType, Map, Structure, Surface};
use noise::{NoiseFn, Perlin};
use physics::{Density, Length, Temperature};

const TERRAIN_FEATURE_SIZE: f64 = 400.0;
const VEGETATION_FEATURE_SIZE: f64 = 150.0;

// Terrain and fluids
const WATER_THRESHOLD: f64 = -0.10;
const BEACH_THRESHOLD: f64 = 0.25;
const GRASS_THRESHOLD: f64 = 0.6;

// Structures
const TREE_THRESHOLD: f64 = 0.42;
const BUSH_THRESHOLD: f64 = 0.15;
const MOUNTAIN_THRESHOLD: f64 = -0.3;

const ELEVATION_SCALE_METERS: f32 = 100.0;

/// Remaps a Perlin's `z` value in `[-1, 1]` to a non-negative elevation.
fn to_elevation(raw: f64) -> Length {
    let normalized = ((raw + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
    Length::from_meters_f32(normalized * ELEVATION_SCALE_METERS)
}

/// A procedurally generated [`Map`], built from Perlin noises.
pub struct GeneratedMap {
    terrain_noise: Perlin,
    structure_noise: Perlin,
}

impl GeneratedMap {
    /// Generates new [`Map`] using provided `seed`.
    pub fn generate(seed: u32) -> Self {
        Self {
            terrain_noise: Perlin::new(seed),
            structure_noise: Perlin::new(seed.wrapping_add(1)),
        }
    }

    /// `z` value of terrain noise at (`x`, `y`), in `[-1, 1]`.
    fn raw_elevation(&self, x: f64, y: f64) -> f64 {
        self.terrain_noise
            .get([x / TERRAIN_FEATURE_SIZE, y / TERRAIN_FEATURE_SIZE])
    }

    /// `z` value of vegetation noise at (`x`, `y`), in `[-1, 1]`.
    fn raw_vegetation(&self, x: f64, y: f64) -> f64 {
        self.structure_noise
            .get([x / VEGETATION_FEATURE_SIZE, y / VEGETATION_FEATURE_SIZE])
    }

    /// Generates new [`Cell`].
    fn generate_cell(&self, x: i32, y: i32) -> Cell {
        let x = x as f64;
        let y = y as f64;

        let raw_elevation = self.raw_elevation(x, y);
        let elevation = to_elevation(raw_elevation);

        let liquid = match raw_elevation {
            z if z < WATER_THRESHOLD => Some(LiquidType::Water),
            _ => None,
        }
        .map(|lt| Liquid::new(lt, to_elevation(WATER_THRESHOLD)));

        let surface = match raw_elevation {
            z if z < BEACH_THRESHOLD => Surface::Beach,
            z if z < GRASS_THRESHOLD => Surface::Grass,
            _ => Surface::Rock,
        };

        let structure = match (surface, self.raw_vegetation(x, y)) {
            (Surface::Grass, z) if z > TREE_THRESHOLD => Some(Structure::Tree),
            (Surface::Grass, z) if z > BUSH_THRESHOLD => Some(Structure::Bush),
            (Surface::Rock, z) if z < MOUNTAIN_THRESHOLD => Some(Structure::Mountain),
            _ => None,
        };

        Cell {
            surface,
            elevation,
            structure,
            liquid,
        }
    }
}

impl Map for GeneratedMap {
    fn cell_at(&self, x: f32, y: f32) -> Cell {
        self.generate_cell(x.floor() as i32, y.floor() as i32)
    }

    fn temperature(&self) -> Temperature {
        Temperature::from_celsius_f32(20.0)
    }

    fn air_density(&self) -> Density {
        Density::from_kilograms_per_cubic_meter_f32(1.2255)
    }

    fn gravity(&self) -> f32 {
        9.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_elevation_maps_smallest_z_value_to_zero() {
        assert_eq!(to_elevation(-1.0), Length::ZERO);
    }

    #[test]
    fn to_elevation_maps_largest_z_value_to_max_elevation() {
        assert_eq!(
            to_elevation(1.0),
            Length::from_meters_f32(ELEVATION_SCALE_METERS)
        );
    }

    #[test]
    fn to_elevation_maps_midpoint_to_half_elevation() {
        assert_eq!(
            to_elevation(0.0),
            Length::from_meters_f32(ELEVATION_SCALE_METERS / 2.0)
        );
    }

    #[test]
    fn to_elevation_clamps_out_of_range() {
        assert_eq!(to_elevation(-2.0), Length::ZERO);
        assert_eq!(
            to_elevation(2.0),
            Length::from_meters_f32(ELEVATION_SCALE_METERS)
        );
    }
}
