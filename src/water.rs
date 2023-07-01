use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

pub mod material;
use material::*;

pub const WATER_SIZE: u32 = 256;
pub const WATER_GRID_SIZE: u32 = 6;

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct WaterSettings {
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
}

impl Default for WaterSettings {
  fn default() -> Self {
    Self {
      height: 1.0,
      amplitude: 1.0,
      clarity: 0.25,
      base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
      deep_color: Color::rgba(0.2, 0.41, 0.54, 1.0),
      shallow_color: Color::rgba(0.45, 0.78, 0.81, 1.0),
      edge_scale: 0.1,
      edge_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
      update_materials: true,
      spawn_tiles: Some(UVec2::new(WATER_GRID_SIZE, WATER_GRID_SIZE)),
    }
  }
}

#[derive(Bundle, Default)]
pub struct WaterBundle {
  pub name: Name,
  #[bundle]
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
  #[bundle]
  pub mesh: MaterialMeshBundle<WaterMaterial>,
}

impl WaterTileBundle {
  pub fn new(
    mesh: Handle<Mesh>,
    material: Handle<WaterMaterial>,
    height: f32,
    offset: Vec2,
  ) -> Self {
    Self {
      name: Name::new(format!("Water Tile {}x{}", offset.x, offset.y)),
      tile: WaterTile {
        offset,
      },
      mesh: MaterialMeshBundle {
        mesh,
        material,
        transform: Transform::from_xyz(offset.x, height, offset.y),
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
  mut materials: ResMut<Assets<WaterMaterial>>,
) {
  let grid = match settings.spawn_tiles {
    Some(grid) => grid,
    None => {
      return;
    }
  };
  let water_height = settings.height;
  // Generate mesh for water.
  let mesh: Handle<Mesh> = meshes.add(
    shape::Plane {
      size: WATER_SIZE as f32,
      subdivisions: WATER_SIZE as u32 / 10,
    }
    .into(),
  );

  commands
    .spawn(WaterBundle {
      name: Name::new("Water"),
      ..default()
    })
    .with_children(|parent| {
      let offset = (WATER_SIZE * WATER_GRID_SIZE) as f32 / 2.0;
      for x in 0..grid.x {
        for y in 0..grid.y {
          let x = (x * WATER_SIZE) as f32 - offset;
          let y = (y * WATER_SIZE) as f32 - offset;
          let tile_offset = Vec2::new(x, y);
          // Water material.
          let material = materials.add(WaterMaterial {
            amplitude: settings.amplitude,
            base_color: settings.base_color,
            clarity: settings.clarity,
            deep_color: settings.deep_color,
            shallow_color: settings.shallow_color,
            edge_color: settings.edge_color,
            edge_scale: settings.edge_scale,
            coord_offset: tile_offset,
            coord_scale: Vec2::new(WATER_SIZE as f32, WATER_SIZE as f32),
            ..default()
          });

          parent.spawn((
            WaterTileBundle::new(mesh.clone(), material, water_height, tile_offset),
            NotShadowCaster,
          ));
        }
      }
    });
}

fn update_materials(settings: Res<WaterSettings>, mut materials: ResMut<Assets<WaterMaterial>>) {
  if !settings.update_materials {
    // Don't update water materials.
    return;
  }
  for (_, mat) in materials.iter_mut() {
    mat.amplitude = settings.amplitude;
    mat.base_color = settings.base_color;
    mat.clarity = settings.clarity;
    mat.deep_color = settings.deep_color;
    mat.shallow_color = settings.shallow_color;
    mat.edge_color = settings.edge_color;
    mat.edge_scale = settings.edge_scale;
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<WaterSettings>()
      .register_type::<WaterSettings>()
      .add_plugin(WaterMaterialPlugin)
      .add_startup_system(setup_water)
      .add_system(update_materials.run_if(resource_changed::<WaterSettings>()));
  }
}
