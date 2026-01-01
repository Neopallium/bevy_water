//! Showcases simple dynamic ocean material.

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::{input::common_conditions, prelude::*};

use bevy_water::*;

#[cfg(feature = "easings")]
use bevy_easings::{EaseMethod, EasingType};

const WATER_HEIGHT: f32 = 20.0;

fn main() {
  let mut app = App::new();

  let app = app
    .add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    });

  #[cfg(feature = "easings")]
  {
    app.insert_resource(WaterHeightEasingSettings {
      height_easing_all_tiles: true,
      height_easing_method: EaseMethod::Linear,
      height_easing_type: EasingType::Once {
        duration: std::time::Duration::from_secs(5),
      },
    });
  }

  app
    .add_plugins(WaterPlugin)
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      toggle_wave_height.run_if(common_conditions::input_just_pressed(KeyCode::KeyH)),
    );

  app.run();
}

// Toggle wave height between two values.
fn toggle_wave_height(mut settings: ResMut<WaterSettings>) {
  if settings.height == WATER_HEIGHT {
    settings.height = WATER_HEIGHT + 5.0;
  } else {
    settings.height = WATER_HEIGHT;
  }
}

/// set up a simple 3D scene
fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // wall
  commands.spawn((
    Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(5.0, 5.0, 0.1))))),
    MeshMaterial3d(materials.add(Color::srgb(0.5, 0.3, 0.3))),
    Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
  ));
  // cube
  commands.spawn((
    Mesh3d(meshes.add(Cuboid::from_length(1.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
    Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
  ));
  // light
  commands.spawn((
    PointLight {
      intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
      shadows_enabled: true,
      ..default()
    },
    Transform::from_xyz(4.0, WATER_HEIGHT + 8.0, 4.0),
  ));

  // camera
  let mut cam = commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(-20.0, WATER_HEIGHT + 5.0, 20.0)
      .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
  ));
  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }

  cam.insert(Name::new("Camera"));
}
