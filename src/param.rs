use bevy::{ecs::system::SystemParam, math::Vec3Swizzles, prelude::*};

use crate::{water::WaterSettings, wave::get_wave_height_2d};

/// A system parameter used to calculate wave height and point based on global WaterSettings and Time resources.
#[derive(SystemParam)]
pub struct WaterParam<'w> {
  pub settings: Res<'w, WaterSettings>,
  pub time: Res<'w, Time>,
}

impl<'w> WaterParam<'w> {
  /// Calculates the height of the waves at the given position.
  ///
  /// # Arguments
  ///
  /// * `position` - The global position at which to calculate the wave height. Use your entity's `GlobalTransform` to get the world position.
  ///
  /// # Returns
  ///
  /// The height of the waves at the given global position.
  pub fn wave_height(&self, position: Vec3) -> f32 {
    let time = self.time.elapsed_secs_wrapped();
    self.settings.height
      + self.settings.amplitude
        * get_wave_height_2d(time, position.xz(), self.settings.water_quality.into())
  }

  /// Calculates the point of the waves at the given position.
  ///
  /// # Arguments
  ///
  /// * `position` - The global position at which to calculate the wave point. Use your entity's `GlobalTransform` to get the world position.
  ///
  /// # Returns
  ///
  /// A point on the surface of the waves at the given global position.
  pub fn wave_point(&self, mut position: Vec3) -> Vec3 {
    position.y = self.wave_height(position);
    position
  }

  /// Calculates the normal vector for a given point on the water surface.
  ///
  /// # Arguments
  ///
  /// * `position` - The global position at which to calculate the wave surface. Use your entity's `GlobalTransform` to get the world position.
  ///
  /// # Returns
  ///
  /// A `Vec3` representing the normal vector at the given point.
  ///
  /// # Details
  ///
  /// Uses first order forward difference with step size 1 to calculate the change in wave position.
  pub fn wave_normal(&self, position: Vec3) -> Vec3 {
    let h = self.wave_height(position);
    let h_dx = self.wave_height(position + Vec3::X);
    let h_dz = self.wave_height(position + Vec3::Z);

    // negative cross product of the change in wave position
    Vec3::new(h - h_dx, 1., h - h_dz).normalize()
  }
}
