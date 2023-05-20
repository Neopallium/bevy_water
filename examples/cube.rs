use bevy::pbr::NotShadowCaster;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::{input::common_conditions, prelude::*};

use bevy_spectator::*;

use bevy_water::material::WaterMaterial;
use bevy_water::*;

const CUBE_SIZE: f32 = 10.0;

fn main() {

  App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
      // Tell the asset server to watch for asset changes on disk:
      watch_for_changes: true,
      ..default()
    }))
    .add_plugin(SpectatorPlugin) // Simple movement for this example
    .insert_resource(WaterSettings {
      spawn_tiles: None,
      ..default()
    })
    .add_plugin(WaterPlugin)
    // Wireframe
    .add_plugin(WireframePlugin)
    .add_startup_system(setup)
    .add_system(toggle_wireframe.run_if(common_conditions::input_just_pressed(KeyCode::R)))
    .run();
}

fn toggle_wireframe(
  mut show_wireframe: Local<bool>,
  query: Query<Entity, With<Handle<Mesh>>>,
  mut commands: Commands,
) {
  // Update flag.
  *show_wireframe = !*show_wireframe;

  for entity in query.iter() {
    let mut entity = commands.entity(entity);
    if *show_wireframe {
      entity.insert(Wireframe);
    } else {
      entity.remove::<Wireframe>();
    }
  }
}

/// Setup water.
fn setup(
  mut commands: Commands,
  settings: Res<WaterSettings>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<WaterMaterial>>,
) {
  // Mesh for water.
  let mesh: Handle<Mesh> = meshes.add(
    shape::Cube {
      size: CUBE_SIZE,
    }
    .into(),
  );
  // Water material.
  let material = materials.add(WaterMaterial {
    amplitude: settings.amplitude,
    coord_scale: Vec2::new(256.0, 256.0),
    ..default()
  });

  commands
    .spawn((
      Name::new(format!("Water world")),
      MaterialMeshBundle {
        mesh,
        material,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
      },
      NotShadowCaster,
    ));

  // light
  commands.spawn(PointLightBundle {
    transform: Transform::from_xyz(4.0, CUBE_SIZE + 8.0, 4.0),
    point_light: PointLight {
      intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // camera
  commands.spawn((Camera3dBundle {
    transform: Transform::from_xyz(-40.0, CUBE_SIZE + 5.0, 0.0)
      .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ..default()
  },
    Spectator,
  ));
}
