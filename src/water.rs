use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

pub mod material;
use material::*;

pub const WATER_SIZE: u32 = 256;
pub const WATER_GRID_SIZE: u32 = 6;

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct WaterSettings {
  pub height: f32,
  pub amplitude: f32,
  pub update_materials: bool,
  pub spawn_tiles: Option<UVec2>,
}

impl Default for WaterSettings {
  fn default() -> Self {
    Self {
      height: 1.0,
      amplitude: 1.0,
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
      subdivisions: WATER_SIZE as u32 - 1,
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
