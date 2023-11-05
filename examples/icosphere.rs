#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::pbr::NotShadowCaster;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::{input::common_conditions, prelude::*};

#[cfg(feature = "panorbit")]
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

use bevy_water::material::{WaterMaterial, StandardWaterMaterial};
use bevy_water::*;

const RADIUS: f32 = 10.0;

fn main() {

  let mut app = App::new();

  app.add_plugins(DefaultPlugins);

  // Simple pan/orbit camera.
  #[cfg(feature = "panorbit")]
  app.add_plugins(PanOrbitCameraPlugin);

  app.insert_resource(WaterSettings {
      amplitude: 0.4,
      spawn_tiles: None,
      ..default()
    })
    .add_plugins(WaterPlugin)
    // Wireframe
    .add_plugins(WireframePlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, toggle_wireframe.run_if(common_conditions::input_just_pressed(KeyCode::R)));

  app.run();
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
  mut water_materials: ResMut<Assets<StandardWaterMaterial>>,
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
  let material = water_materials.add(StandardWaterMaterial {
    base: default(),
    extension: WaterMaterial {
      amplitude: settings.amplitude,
      clarity: 0.05,
      coord_scale: Vec2::new(256.0, 256.0),
      ..default()
    },
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
      radius: RADIUS - 0.8,
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
  #[cfg(not(feature = "panorbit"))]
  let mut cam = commands.spawn(Camera3dBundle {
    transform: Transform::from_xyz(0.0, RADIUS + 15.0, 0.0)
      .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ..default()
  });
  #[cfg(feature = "panorbit")]
  let mut cam = commands.spawn((Camera3dBundle {
    ..default()
  },
    PanOrbitCamera {
      focus: Vec3::new(0.0, 0.0, 0.0),
      radius: Some(RADIUS + 15.0),
      alpha: Some(3.14),
      beta: Some(0.0),
      ..default()
    },
  ));
  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }
  // This is just to keep the compiler happy when not using `depth_prepass` feature.
  cam.insert(Name::new("Camera"));
}
