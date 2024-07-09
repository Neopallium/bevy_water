use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::*;

pub mod material;
use material::*;

pub const WATER_SIZE: u32 = 256;
pub const WATER_HALF_SIZE: f32 = WATER_SIZE as f32 / 2.0;
pub const WATER_GRID_SIZE: u32 = 6;

#[derive(Debug, Clone, Reflect)]
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
    }
  }
}

#[derive(Bundle, Default)]
pub struct WaterBundle {
  pub name: Name,
  pub spatial: SpatialBundle,
}

#[derive(Component, Default)]
pub struct WaterTile {
  pub offset: Vec2,
}

#[derive(Bundle, Default)]
pub struct WaterTileBundle {
  pub name: Name,
  pub tile: WaterTile,
  pub mesh: MaterialMeshBundle<StandardWaterMaterial>,
}

impl WaterTileBundle {
  pub fn new(
    mesh: Handle<Mesh>,
    material: Handle<StandardWaterMaterial>,
    height: f32,
    offset: Vec2,
  ) -> Self {
    // The tile position is based on the center of the tile,
    // so we need to add `WATER_HALF_SIZE` so the tile corner absolute position
    // will match `coord` in the water shader.
    let tile_pos = offset + WATER_HALF_SIZE;
    Self {
      name: Name::new(format!("Water Tile {}x{}", offset.x, offset.y)),
      tile: WaterTile { offset },
      mesh: MaterialMeshBundle {
        mesh,
        material,
        transform: Transform::from_xyz(tile_pos.x, height, tile_pos.y),
        ..default()
      },
    }
  }
}

/// Setup water.
fn setup_water(
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
  let mesh: Handle<Mesh> = meshes.add(plane_builder);

  commands
    .spawn(WaterBundle {
      name: Name::new("Water"),
      ..default()
    })
    .with_children(|parent| {
      let grid_center = (WATER_SIZE * WATER_GRID_SIZE) as f32 / 2.0;
      for x in 0..grid.x {
        for y in 0..grid.y {
          let x = (x * WATER_SIZE) as f32 - grid_center;
          let y = (y * WATER_SIZE) as f32 - grid_center;
          // UV starts at (0,0) at the corner.
          let coord_offset = Vec2::new(x, y);
          // Water material.
          let material = materials.add(StandardWaterMaterial {
            base: StandardMaterial {
              base_color: settings.base_color,
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
              quality: settings.water_quality.clone().into(),
              ..default()
            },
          });

          let mut tile_bundle = parent.spawn((
            WaterTileBundle::new(mesh.clone(), material, water_height, coord_offset),
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

fn update_materials(
  settings: Res<WaterSettings>,
  mut materials: ResMut<Assets<StandardWaterMaterial>>,
) {
  if !settings.update_materials {
    // Don't update water materials.
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
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<WaterSettings>()
      .register_type::<WaterSettings>()
      .add_plugins(WaterMaterialPlugin)
      .add_systems(Startup, setup_water)
      .add_systems(
        Update,
        update_materials.run_if(resource_changed::<WaterSettings>),
      );
  }
}
