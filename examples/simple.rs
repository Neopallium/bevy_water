//! Showcases simple dynamic ocean material.

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::prelude::*;

use bevy_water::*;

const WATER_HEIGHT: f32 = 20.0;

fn main() {
  let mut app = App::new();

  app.add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    })
    .add_plugins(WaterPlugin)
    .add_systems(Startup, setup);

  app.run();
}

/// set up a simple 3D scene
fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // wall
  commands.spawn(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 5.0, 0.1))),
    material: materials.add(Color::rgb(0.5, 0.3, 0.3)),
    transform: Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
    ..default()
  });
  // cube
  commands.spawn(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
    transform: Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
    ..default()
  });
  // light
  commands.spawn(PointLightBundle {
    transform: Transform::from_xyz(4.0, WATER_HEIGHT + 8.0, 4.0),
    point_light: PointLight {
      intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // camera
  let mut cam = commands.spawn(Camera3dBundle {
    transform: Transform::from_xyz(-20.0, WATER_HEIGHT + 5.0, 20.0)
      .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
    ..default()
  });
  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }

  cam.insert(Name::new("Camera"));
}
