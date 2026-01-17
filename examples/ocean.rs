//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

use bevy::light::NotShadowCaster;
use bevy::mesh::*;
use bevy::prelude::*;

mod pirates;
use pirates::*;

fn main() {
  let mut app = pirates_app("Simple ocean with ships");

  // Setup
  app.add_systems(
    Startup,
    (setup_simple_ocean, setup_camera, setup_simple_ships),
  );

  app.run();
}

/// Setup a simple ocean.
fn setup_simple_ocean(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // Terrain material.
  let material = MeshMaterial3d(materials.add(StandardMaterial {
    base_color: Color::srgba_u8(177, 168, 132, 255),
    perceptual_roughness: 0.6,
    metallic: 0.6,
    reflectance: 0.8,
    ..default()
  }));

  // Spawn simple terrain plane.
  commands.spawn((
    Name::new(format!("Terrain")),
    Mesh3d(meshes.add(PlaneMeshBuilder::from_length(256.0 * 6.0))),
    material.clone(),
    Transform::from_xyz(0.0, -5.0, 0.0),
    NotShadowCaster,
  ));
  // Spawn fake island.
  commands.spawn((
    Name::new(format!("Fake island")),
    Mesh3d(meshes.add(Sphere::new(2.0))),
    material.clone(),
    Transform::from_xyz(-30.0, -10.0, -30.0).with_scale(Vec3::new(30.0, 6.5, 30.0)),
    NotShadowCaster,
  ));
}

/// Setup some simple ships.
fn setup_simple_ships(mut commands: Commands, asset_server: Res<AssetServer>) {
  // Spawn ships.
  let scene = SceneRoot(asset_server.load("models/Kenney_pirate/ship_dark.gltf#Scene0"));
  let ship = Ship::new(-0.400, -3.8, 2.5, -1.4, 1.4);

  // "Randomly" place the ships.
  for x in 1..18 {
    let f = (x as f32) * 1.20;
    let f2 = ((x % 6) as f32) * -10.90;
    commands.spawn((
      ship.clone(),
      Name::new(format!("Ship {x}")),
      scene.clone(),
      Transform::from_xyz(-20.0 + (f * 7.8), 0.0, 20.0 + f2)
        .with_rotation(Quat::from_rotation_y(f)),
    ));
  }
}
