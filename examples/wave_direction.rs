//! Example demonstrating wave direction control with keyboard input.
//!
//! Controls:
//! - Arrow keys: Change wave direction
//! - R: Reset to default direction

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::prelude::*;
use bevy_water::*;

const WATER_HEIGHT: f32 = 1.0;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      wave_direction: Vec2::new(1.0, 0.0),
      ..default()
    })
    .add_plugins(WaterPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, (handle_input, update_direction_display))
    .run();
}

fn handle_input(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<WaterSettings>) {
  let mut dir = settings.wave_direction;

  // Rotate direction with arrow keys
  if keys.just_pressed(KeyCode::ArrowLeft) {
    dir = rotate_vec2(dir, 45.0_f32.to_radians());
  }
  if keys.just_pressed(KeyCode::ArrowRight) {
    dir = rotate_vec2(dir, -45.0_f32.to_radians());
  }
  if keys.just_pressed(KeyCode::ArrowUp) {
    dir = rotate_vec2(dir, 22.5_f32.to_radians());
  }
  if keys.just_pressed(KeyCode::ArrowDown) {
    dir = rotate_vec2(dir, -22.5_f32.to_radians());
  }

  // Reset to default
  if keys.just_pressed(KeyCode::KeyR) {
    dir = Vec2::new(1.0, 0.0);
  }

  if dir != settings.wave_direction {
    settings.wave_direction = dir;
  }
}

fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
  let cos = angle.cos();
  let sin = angle.sin();
  Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

#[derive(Component)]
struct DirectionText;

fn update_direction_display(
  settings: Res<WaterSettings>,
  mut query: Query<&mut Text, With<DirectionText>>,
) {
  if let Ok(mut text) = query.single_mut() {
    let dir = settings.wave_direction;
    let angle = dir.y.atan2(dir.x).to_degrees();
    *text = Text::new(format!(
      "Wave Direction: ({:.2}, {:.2})\nAngle: {:.1} deg\n\nControls:\nLeft/Right: Rotate 45 deg\nUp/Down: Rotate 22.5 deg\nR: Reset",
      dir.x, dir.y, angle
    ));
  }
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // Reference cube to see wave motion
  commands.spawn((
    Mesh3d(meshes.add(Cuboid::from_length(2.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
    Transform::from_xyz(0.0, WATER_HEIGHT - 1.0, 0.0),
  ));

  // Directional light
  commands.spawn((
    DirectionalLight {
      illuminance: 10000.0,
      shadows_enabled: true,
      ..default()
    },
    Transform::from_xyz(4.0, 10.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
  ));

  // Camera
  let mut cam = commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(-30.0, WATER_HEIGHT + 20.0, 30.0)
      .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
  ));
  #[cfg(feature = "depth_prepass")]
  {
    cam.insert(DepthPrepass);
  }

  // UI text
  commands.spawn((
    DirectionText,
    Text::new("Wave Direction: (1.00, 0.00)\nAngle: 0.0 deg\n\nControls:\nLeft/Right: Rotate 45 deg\nUp/Down: Rotate 22.5 deg\nR: Reset"),
    Node {
      position_type: PositionType::Absolute,
      top: Val::Px(10.0),
      left: Val::Px(10.0),
      ..default()
    },
  ));
}
