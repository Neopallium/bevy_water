use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

pub mod material;
use material::*;

pub const WATER_SIZE: u16 = 256;
pub const WATER_QUAD_SIZE: u16 = 16;
pub const WATER_GRID_SIZE: u16 = 6;

#[derive(Resource, Clone, Debug)]
pub struct WaterSettings {
  pub height: f32,
}

impl Default for WaterSettings {
  fn default() -> Self {
    Self { height: 1.0 }
  }
}

#[derive(Bundle, Default)]
pub struct WaterBundle {
  name: Name,
  #[bundle]
  spatial: SpatialBundle,
}

#[derive(Bundle, Default)]
pub struct WaterTileBundle {
  name: Name,
  #[bundle]
  mesh: MaterialMeshBundle<WaterMaterial>,
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
  let water_height = settings.height;
  // Generate mesh for water.
  let mesh = meshes.add(crate::generate::grid_mesh(
    WATER_SIZE / WATER_QUAD_SIZE,
    WATER_QUAD_SIZE as f32,
    1.0,
  ));

  commands
    .spawn(WaterBundle {
      name: Name::new("Water"),
      ..default()
    })
    .with_children(|parent| {
      let offset = (WATER_SIZE * WATER_GRID_SIZE) as f32 / 2.0;
      for x in 0..WATER_GRID_SIZE {
        for y in 0..WATER_GRID_SIZE {
          let x = (x * WATER_SIZE) as f32 - offset;
          let y = (y * WATER_SIZE) as f32 - offset;
          // Water material. TODO: re-use?
          let material = materials.add(WaterMaterial::new());

          parent.spawn((
            WaterTileBundle::new(mesh.clone(), material, water_height, Vec2::new(x, y)),
            NotShadowCaster,
          ));
        }
      }
    });
}

#[derive(Default, Clone, Debug)]
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<WaterSettings>()
      .add_plugin(WaterMaterialPlugin)
      .add_startup_system(setup_water);
  }
}
