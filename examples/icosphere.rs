use bevy::pbr::NotShadowCaster;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::{input::common_conditions, prelude::*};

use bevy_spectator::*;

use bevy_water::material::WaterMaterial;
use bevy_water::*;

const RADIUS: f32 = 10.0;

fn main() {

  App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
      // Tell the asset server to watch for asset changes on disk:
      watch_for_changes: true,
      ..default()
    }))
    .add_plugin(SpectatorPlugin) // Simple movement for this example
    .insert_resource(WaterSettings {
      amplitude: 0.4,
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
  mut water_materials: ResMut<Assets<WaterMaterial>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // Mesh for water.
  let mesh: Handle<Mesh> = meshes.add(
    shape::Icosphere {
      radius: RADIUS,
      subdivisions: 15,
    }
    .try_into().expect("Icosphere"),
  );
  // Water material.
  let material = water_materials.add(WaterMaterial {
    base_color: Color::rgba(0.01, 0.03, 0.05, 0.99),
    amplitude: settings.amplitude,
    coord_scale: Vec2::new(256.0, 256.0),
    ..default()
  });

  // Spawn water entity.
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

  // Mesh for terrain.
  let mesh: Handle<Mesh> = meshes.add(
    shape::Icosphere {
      radius: RADIUS - 0.4,
      subdivisions: 15,
    }
    .try_into().expect("Icosphere"),
  );
  // Terrain material.
  let material = materials.add(StandardMaterial {
    base_color: Color::OLIVE,
    ..default()
  });

  // Spawn planet entity.
  commands
    .spawn((
      Name::new(format!("Planet terrain")),
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
    transform: Transform::from_xyz(4.0, RADIUS + 8.0, 4.0),
    point_light: PointLight {
      intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // camera
  commands.spawn((Camera3dBundle {
    transform: Transform::from_xyz(-40.0, RADIUS + 5.0, 0.0)
      .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ..default()
  },
    Spectator,
  ));
}
