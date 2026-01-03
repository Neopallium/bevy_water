use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::*;
use bevy::prelude::*;
#[cfg(feature = "easings")]
pub use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingType, EasingsPlugin};

pub mod material;
use material::*;

use crate::{mix2d, sample_directional_wave_blended, smoothstep};

/// Component for tracking wave direction using dual-direction crossfade blending.
///
/// Instead of rotating waves (which looks unnatural), this crossfades between
/// two independent wave patterns: the old direction fading out while the new
/// direction fades in. This matches how real water responds to changing wind.
#[derive(Component, Clone, Copy, Debug)]
pub struct WaveDirection {
  /// Direction A (fading out).
  dir_a: Vec2,
  /// Direction B (fading in).
  dir_b: Vec2,
  /// Blend factor: 0 = fully A, 1 = fully B.
  blend: f32,
  /// Blend duration in seconds.
  blend_duration: f32,
  /// Per-tile offset for desynchronized transitions (0.0-0.3 typical).
  pub tile_offset: f32,
}

impl Default for WaveDirection {
  fn default() -> Self {
    Self::new(Vec2::new(1.0, 2.0))
  }
}

impl WaveDirection {
  /// Create a new WaveDirection from a direction vector.
  pub fn new(direction: Vec2) -> Self {
    let normalized = direction.normalize_or_zero();
    Self {
      dir_a: normalized,
      dir_b: normalized,
      blend: 1.0, // Start fully blended (no transition)
      blend_duration: 2.0,
      tile_offset: 0.0,
    }
  }

  /// Create a new WaveDirection with custom blend duration.
  pub fn with_duration(direction: Vec2, blend_duration: f32) -> Self {
    let normalized = direction.normalize_or_zero();
    Self {
      dir_a: normalized,
      dir_b: normalized,
      blend: 1.0,
      blend_duration,
      tile_offset: 0.0,
    }
  }

  /// Set a new target direction. Starts crossfade from current state.
  pub fn set_target(&mut self, target: Vec2) {
    let normalized = target.normalize_or_zero();

    // Don't restart if already heading to this target
    if (self.dir_b - normalized).length_squared() < 0.001 {
      return;
    }

    // Snapshot current blended state as "old" direction
    self.dir_a = self.current_blended();
    self.dir_b = normalized;
    self.blend = 0.0;
  }

  /// Set the blend duration in seconds.
  pub fn set_duration(&mut self, duration: f32) {
    self.blend_duration = duration;
  }

  /// Update the blend. Call this every frame.
  pub fn update(&mut self, dt: f32) {
    if self.blend < 1.0 {
      self.blend = (self.blend + dt / self.blend_duration).min(1.0);
    }
  }

  /// Get direction A (fading out).
  pub fn dir_a(&self) -> Vec2 {
    self.dir_a
  }

  /// Get direction B (fading in).
  pub fn dir_b(&self) -> Vec2 {
    self.dir_b
  }

  /// Get the raw blend factor (0-1).
  pub fn blend(&self) -> f32 {
    self.blend
  }

  /// Get the effective blend with tile offset applied.
  pub fn effective_blend(&self) -> f32 {
    (self.blend - self.tile_offset).clamp(0.0, 1.0)
  }

  /// Get the current blended direction (for physics/CPU calculations).
  pub fn current_blended(&self) -> Vec2 {
    let blend = smoothstep(0.0, 0.85, self.blend);
    mix2d(self.dir_a, self.dir_b, blend).normalize_or_zero()
  }

  /// Get the effective blended direction with tile offset applied (for GPU/shader).
  pub fn effective_direction(&self) -> Vec2 {
    self
      .dir_a
      .lerp(self.dir_b, self.effective_blend())
      .normalize_or_zero()
  }

  /// Check if the blend is complete.
  pub fn is_settled(&self) -> bool {
    self.blend >= 1.0
  }
}

pub const WATER_SIZE: u32 = 256;
pub const WATER_HALF_SIZE: f32 = WATER_SIZE as f32 / 2.0;
pub const WATER_GRID_SIZE: u32 = 6;

/// Global wave transition state for physics calculations.
/// This mirrors the shader's wave direction blending state.
#[derive(Resource, Clone, Copy, Debug)]
pub struct GlobalWaveState {
  /// Direction A (fading out).
  pub dir_a: Vec2,
  /// Direction B (fading in).
  pub dir_b: Vec2,
  /// Blend factor: 0 = fully A, 1 = fully B.
  pub blend: f32,
}

impl Default for GlobalWaveState {
  fn default() -> Self {
    let default_dir = Vec2::new(1.0, 2.0).normalize();
    Self {
      dir_a: default_dir,
      dir_b: default_dir,
      blend: 1.0,
    }
  }
}

impl GlobalWaveState {
  /// Get the blended wave height using dual-sample crossfade (matches High/Ultra shader).
  pub fn blended_height(&self, time: f32, p: Vec2, amplitude: f32) -> f32 {
    sample_directional_wave_blended(time, p, self.dir_a, self.dir_b, self.blend, u32::MAX)
      * amplitude
  }

  /// Get the current blended direction.
  pub fn current_direction(&self) -> Vec2 {
    self.dir_a.lerp(self.dir_b, self.blend).normalize_or_zero()
  }
}

#[derive(Debug, Clone, Copy, Reflect)]
#[repr(u32)]
pub enum WaterQuality {
  Basic,
  Medium,
  High,
  Ultra,
}

impl Into<u32> for WaterQuality {
  fn into(self) -> u32 {
    match self {
      WaterQuality::Basic => 1,
      WaterQuality::Medium => 2,
      WaterQuality::High => 3,
      WaterQuality::Ultra => 4,
    }
  }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct WaterSettings {
  /// StandardMaterial setting.
  pub alpha_mode: AlphaMode,
  /// Base water height.
  pub height: f32,
  /// Wave amplitude.
  pub amplitude: f32,
  /// The `StandardMaterial` base_color field.  This is the base color of the water.
  /// When using `DepthPrepass` it is recommended to use the `deep_color` and `shallow_color` fields.
  pub base_color: Color,
  /// Water clarity, 0.0 = invisible.
  pub clarity: f32,
  /// Water color at deepest level.
  pub deep_color: Color,
  /// Water color at shallow areas.
  pub shallow_color: Color,
  /// Scale of the water edge effect.
  pub edge_scale: f32,
  /// Color of the edge effect.
  pub edge_color: Color,
  /// Update all `WaterMaterial`s from the global `WaterSettings` resource when it changes.
  ///
  /// This allows easy editing all materials.
  pub update_materials: bool,
  /// During startup, spawn a 2d grid of water tiles.
  pub spawn_tiles: Option<UVec2>,
  /// Water quality
  ///
  pub water_quality: WaterQuality,
  /// Wave movement direction.
  pub wave_direction: Vec2,
  /// Duration in seconds for wave direction crossfade transitions.
  /// Default: 2.0. Longer = more gradual, imperceptible transitions.
  pub wave_direction_blend_duration: f32,
}

impl Default for WaterSettings {
  fn default() -> Self {
    Self {
      #[cfg(not(feature = "ssr"))]
      alpha_mode: AlphaMode::Blend,
      #[cfg(feature = "ssr")]
      alpha_mode: AlphaMode::Opaque,
      height: 1.0,
      amplitude: 1.0,
      clarity: 0.25,
      base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
      deep_color: Color::srgba(0.2, 0.41, 0.54, 1.0),
      shallow_color: Color::srgba(0.45, 0.78, 0.81, 1.0),
      edge_scale: 0.1,
      edge_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
      update_materials: true,
      spawn_tiles: Some(UVec2::new(WATER_GRID_SIZE, WATER_GRID_SIZE)),
      water_quality: WaterQuality::Ultra,
      wave_direction: Vec2::new(1.0, 2.0),
      wave_direction_blend_duration: 2.0,
    }
  }
}

#[derive(Resource, Clone)]
#[cfg(feature = "easings")]
pub struct WaterHeightEasingSettings {
  /// Enable height easing for all water tiles.
  pub height_easing_all_tiles: bool,
  /// Easing method for `height` changes.
  pub height_easing_method: EaseMethod,
  /// Easing type for `height` changes.
  pub height_easing_type: EasingType,
}

#[cfg(feature = "easings")]
impl Default for WaterHeightEasingSettings {
  fn default() -> Self {
    Self {
      height_easing_all_tiles: true,
      height_easing_method: EaseMethod::Linear,
      height_easing_type: EasingType::Once {
        duration: std::time::Duration::from_secs(1),
      },
    }
  }
}

#[derive(Component, Default)]
#[require(Transform, Visibility)]
pub struct WaterTiles;

#[derive(Component, Default)]
#[require(Mesh3d, MeshMaterial3d<StandardWaterMaterial>, Transform, Visibility)]
pub struct WaterTile {
  pub offset: Vec2,
}

impl WaterTile {
  pub fn new(height: f32, offset: Vec2) -> (Self, Name, Transform) {
    // The tile position is based on the center of the tile,
    // so we need to add `WATER_HALF_SIZE` so the tile corner absolute position
    // will match `coord` in the water shader.
    let tile_pos = offset + WATER_HALF_SIZE;
    (
      WaterTile { offset },
      Name::new(format!("Water Tile {}x{}", offset.x, offset.y)),
      Transform::from_xyz(tile_pos.x, height, tile_pos.y),
    )
  }
}

/// Setup water.
pub fn setup_water(
  mut commands: Commands,
  settings: Res<WaterSettings>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardWaterMaterial>>,
) {
  let grid = match settings.spawn_tiles {
    Some(grid) => grid,
    None => {
      return;
    }
  };
  let water_height = settings.height;
  let mut plane_builder = PlaneMeshBuilder::from_length(WATER_SIZE as f32);
  plane_builder = match settings.water_quality {
    WaterQuality::Basic => plane_builder,
    WaterQuality::Medium => plane_builder,
    WaterQuality::High => plane_builder.subdivisions(WATER_SIZE as u32 / 16),
    WaterQuality::Ultra => plane_builder.subdivisions(WATER_SIZE as u32 / 4),
  };

  // Generate mesh for water.
  let mesh = Mesh3d(meshes.add(plane_builder));

  commands
    .spawn((WaterTiles, Name::new("Water")))
    .with_children(|parent| {
      let grid_center_x = (WATER_SIZE * grid.x) as f32 / 2.0;
      let grid_center_y = (WATER_SIZE * grid.y) as f32 / 2.0;
      for x in 0..grid.x {
        for y in 0..grid.y {
          let x = (x * WATER_SIZE) as f32 - grid_center_x;
          let y = (y * WATER_SIZE) as f32 - grid_center_y;
          // UV starts at (0,0) at the corner.
          let coord_offset = Vec2::new(x, y);
          // Water material.
          // Per-tile offset for desynchronized transitions (based on tile position)
          let tile_hash =
            ((x as i32).wrapping_mul(73856093) ^ (y as i32).wrapping_mul(19349663)) as f32;
          let tile_offset = (tile_hash.abs() % 1000.0) / 1000.0 * 0.3; // 0-0.3 range

          let normalized_dir = settings.wave_direction.normalize_or_zero();
          let material = MeshMaterial3d(materials.add(StandardWaterMaterial {
            base: StandardMaterial {
              base_color: settings.base_color,
              alpha_mode: settings.alpha_mode,
              #[cfg(not(feature = "ssr"))]
              perceptual_roughness: 0.22,
              #[cfg(feature = "ssr")]
              perceptual_roughness: 0.0,
              ..default()
            },
            extension: WaterMaterial {
              amplitude: settings.amplitude,
              clarity: settings.clarity,
              deep_color: settings.deep_color,
              shallow_color: settings.shallow_color,
              edge_color: settings.edge_color,
              edge_scale: settings.edge_scale,
              coord_offset,
              coord_scale: Vec2::new(WATER_SIZE as f32, WATER_SIZE as f32),
              wave_dir_a: normalized_dir,
              wave_dir_b: normalized_dir,
              wave_blend: 1.0,
              quality: settings.water_quality.into(),
            },
          }));

          let mut wave_dir = WaveDirection::with_duration(
            settings.wave_direction,
            settings.wave_direction_blend_duration,
          );
          wave_dir.tile_offset = tile_offset;

          let mut tile_bundle = parent.spawn((
            WaterTile::new(water_height, coord_offset),
            mesh.clone(),
            material,
            wave_dir,
            NotShadowCaster,
          ));

          match settings.water_quality {
            WaterQuality::Basic | WaterQuality::Medium => {
              tile_bundle.insert(NotShadowReceiver);
            }
            _ => {}
          };
        }
      }
    });
}

#[cfg(feature = "easings")]
pub fn update_water_height(
  mut commands: Commands,
  settings: Res<WaterSettings>,
  easing_settings: Res<WaterHeightEasingSettings>,
  water_transforms: Query<(Entity, &Transform), With<WaterTile>>,
) {
  for (entity, transform) in water_transforms.iter() {
    // Apply height easing if height has changed
    if transform.translation.y != settings.height {
      let target_transform = Transform::from_xyz(
        transform.translation.x,
        settings.height,
        transform.translation.z,
      );

      commands.entity(entity).insert(transform.ease_to(
        target_transform,
        easing_settings.height_easing_method,
        easing_settings.height_easing_type,
      ));
    }
  }
}

pub fn update_materials(
  settings: Res<WaterSettings>,
  mut materials: ResMut<Assets<StandardWaterMaterial>>,
) {
  if !settings.update_materials {
    return;
  }
  for (_, mat) in materials.iter_mut() {
    mat.base.base_color = settings.base_color;
    mat.base.alpha_mode = settings.alpha_mode;
    mat.extension.amplitude = settings.amplitude;
    mat.extension.clarity = settings.clarity;
    mat.extension.deep_color = settings.deep_color;
    mat.extension.shallow_color = settings.shallow_color;
    mat.extension.edge_color = settings.edge_color;
    mat.extension.edge_scale = settings.edge_scale;
    mat.extension.quality = settings.water_quality.into();
  }
}

/// Sync wave direction from WaterSettings to WaveDirection components.
pub fn sync_wave_direction_settings(
  settings: Res<WaterSettings>,
  mut water_tiles: Query<&mut WaveDirection, With<WaterTile>>,
) {
  for mut wave_dir in water_tiles.iter_mut() {
    wave_dir.set_target(settings.wave_direction);
    wave_dir.set_duration(settings.wave_direction_blend_duration);
  }
}

/// Update global wave state from the first tile's WaveDirection (for physics).
pub fn update_global_wave_state(
  mut global_state: ResMut<GlobalWaveState>,
  water_tiles: Query<&WaveDirection, With<WaterTile>>,
) {
  // Use the first tile's state (without tile_offset) as the global state
  if let Some(wave_dir) = water_tiles.iter().next() {
    global_state.dir_a = wave_dir.dir_a();
    global_state.dir_b = wave_dir.dir_b();
    global_state.blend = wave_dir.blend();
  }
}

/// Update wave direction spring simulation each frame.
pub fn update_wave_direction(
  time: Res<Time>,
  mut water_tiles: Query<&mut WaveDirection, With<WaterTile>>,
) {
  let dt = time.delta_secs();
  for mut wave_dir in water_tiles.iter_mut() {
    wave_dir.update(dt);
  }
}

/// Apply wave direction to materials each frame.
pub fn apply_wave_direction(
  mut materials: ResMut<Assets<StandardWaterMaterial>>,
  water_tiles: Query<(&MeshMaterial3d<StandardWaterMaterial>, &WaveDirection)>,
) {
  for (material_handle, wave_dir) in water_tiles.iter() {
    if let Some(mat) = materials.get_mut(&material_handle.0) {
      mat.extension.wave_dir_a = wave_dir.dir_a();
      mat.extension.wave_dir_b = wave_dir.dir_b();
      mat.extension.wave_blend = wave_dir.blend();
    }
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
  fn build(&self, app: &mut App) {
    let app = app
      .init_resource::<WaterSettings>()
      .init_resource::<GlobalWaveState>()
      .register_type::<WaterSettings>()
      .add_plugins(WaterMaterialPlugin)
      .add_systems(Startup, setup_water);

    #[cfg(feature = "easings")]
    {
      app
        .init_resource::<WaterHeightEasingSettings>()
        .add_plugins(EasingsPlugin::default())
        .add_systems(
          Update,
          update_water_height.run_if(resource_changed::<WaterSettings>),
        );
    }

    app.add_systems(
      Update,
      (
        update_materials.run_if(resource_changed::<WaterSettings>),
        sync_wave_direction_settings.run_if(resource_changed::<WaterSettings>),
        update_wave_direction,
        update_global_wave_state,
        apply_wave_direction,
      )
        .chain(),
    );
  }
}
