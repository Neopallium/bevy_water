//! Test water material on meshes with and without UVs.
//!
//! Left: plane with UVs. Right: the same plane with its UVs removed.
//! Both planes use world-space wave coordinates, so the waves should line up across the seam.
//! Behind them is a sphere without UVs.
//!
//! Press `Q` to cycle the water quality (Basic, Medium, High, Ultra).

use bevy::{
  camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
  input::common_conditions,
  light::NotShadowCaster,
  mesh::PlaneMeshBuilder,
  prelude::*,
};

use bevy_water::{material::*, *};

const WATER_HEIGHT: f32 = 1.0;
const PLANE_SIZE: f32 = 64.0;

fn main() {
  App::new()
    .add_plugins((DefaultPlugins, FreeCameraPlugin))
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      // Don't spawn the default water tiles.
      spawn_tiles: None,
      ..default()
    })
    .add_plugins(WaterPlugin)
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      cycle_quality.run_if(common_conditions::input_just_pressed(KeyCode::KeyQ)),
    )
    .run();
}

fn cycle_quality(mut settings: ResMut<WaterSettings>) {
  settings.water_quality = match settings.water_quality {
    WaterQuality::Basic => WaterQuality::Medium,
    WaterQuality::Medium => WaterQuality::High,
    WaterQuality::High => WaterQuality::Ultra,
    WaterQuality::Ultra => WaterQuality::Basic,
  };
  info!("Water quality: {:?}", settings.water_quality);
}

fn water_material(settings: &WaterSettings, center: Vec2, size: f32) -> StandardWaterMaterial {
  StandardWaterMaterial {
    base: StandardMaterial {
      base_color: settings.base_color,
      alpha_mode: settings.alpha_mode,
      perceptual_roughness: 0.22,
      ..default()
    },
    extension: WaterMaterial {
      amplitude: settings.amplitude,
      clarity: settings.clarity,
      deep_color: settings.deep_color,
      shallow_color: settings.shallow_color,
      edge_color: settings.edge_color,
      edge_scale: settings.edge_scale,
      // Map UV (0..1) to world XZ, so meshes with and without UVs use the same wave coords.
      coord_offset: center - Vec2::splat(size / 2.0),
      coord_scale: Vec2::splat(size),
      quality: settings.water_quality.into(),
      ..default()
    },
  }
}

fn setup(
  mut commands: Commands,
  settings: Res<WaterSettings>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut water_materials: ResMut<Assets<StandardWaterMaterial>>,
) {
  let plane: Mesh = PlaneMeshBuilder::from_length(PLANE_SIZE)
    .subdivisions(64)
    .build();
  let mut plane_no_uvs = plane.clone();
  plane_no_uvs.remove_attribute(Mesh::ATTRIBUTE_UV_0);

  // Plane with UVs (left).
  let center = Vec2::new(-PLANE_SIZE / 2.0, 0.0);
  commands.spawn((
    Name::new("Plane with UVs"),
    Mesh3d(meshes.add(plane)),
    MeshMaterial3d(water_materials.add(water_material(&settings, center, PLANE_SIZE))),
    Transform::from_xyz(center.x, WATER_HEIGHT, center.y),
    NotShadowCaster,
  ));

  // Plane without UVs (right).
  let center = Vec2::new(PLANE_SIZE / 2.0, 0.0);
  commands.spawn((
    Name::new("Plane without UVs"),
    Mesh3d(meshes.add(plane_no_uvs)),
    MeshMaterial3d(water_materials.add(water_material(&settings, center, PLANE_SIZE))),
    Transform::from_xyz(center.x, WATER_HEIGHT, center.y),
    NotShadowCaster,
  ));

  // Sphere without UVs (behind).
  let mut sphere: Mesh = Sphere::new(16.0).mesh().uv(64, 32);
  sphere.remove_attribute(Mesh::ATTRIBUTE_UV_0);
  let center = Vec2::new(0.0, -PLANE_SIZE);
  commands.spawn((
    Name::new("Sphere without UVs"),
    Mesh3d(meshes.add(sphere)),
    MeshMaterial3d(water_materials.add(water_material(&settings, center, 32.0))),
    Transform::from_xyz(center.x, WATER_HEIGHT + 16.0, center.y),
    NotShadowCaster,
  ));

  // Light.
  commands.spawn((
    DirectionalLight {
      shadow_maps_enabled: true,
      ..default()
    },
    Transform::from_xyz(1.0, 1.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
  ));

  // Camera.
  commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, WATER_HEIGHT + 30.0, 60.0)
      .looking_at(Vec3::new(0.0, WATER_HEIGHT, -10.0), Vec3::Y),
    FreeCamera::default(),
  ));
}
