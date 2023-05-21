//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::time::Stopwatch;
use bevy::utils::Duration;
use bevy::{app::AppExit, prelude::*};

use bevy_atmosphere::prelude::*;
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use bevy_water::*;

const WATER_HEIGHT: f32 = 1.0;
const SPEED_MIN: f32 = 0.05;
const SPEED_DELTA: f32 = 0.01;
const SPEED_MAX: f32 = 1.0;

fn main() {
  let mut app = App::new();
  app
    // Tell the asset server to watch for asset changes on disk:
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "Pirates".to_string(),
          resolution: (1200., 600.).into(),
          ..Default::default()
        }),
        ..default()
      }).set(AssetPlugin {
      watch_for_changes: true,
      ..default()
    }));

  #[cfg(feature = "debug")]
  app.add_plugin(DebugLinesPlugin::with_depth_test(true))
    .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    // Atmosphere + daylight cycle.
  app.insert_resource(AtmosphereModel::new(Nishita {
      sun_position: Vec3::new(0.0, 1.0, 1.0),
      ..default()
    }))
    // Simple pan/orbit camera.
    .add_plugin(PanOrbitCameraPlugin)
    .add_plugin(AtmospherePlugin)
    .insert_resource(CycleTimer::new(
      Duration::from_millis(1000),
      0.2,
    ))
    .add_system(timer_control)
    .add_system(daylight_cycle)
    // Improve shadows.
    .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4 * 1024 })
    // Water
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    })
    .add_plugin(WaterPlugin)
    // Ship Physics.
    .add_system(update_ships)
    // Setup
    .add_startup_system(setup)
    .add_system(handle_quit)
    // Wireframe
    .add_plugin(WireframePlugin)
    .init_resource::<UiState>()
    .add_system(toggle_wireframe)
    .run();
}

fn handle_quit(input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
  if input.pressed(KeyCode::Q) {
    exit.send(AppExit);
  }
}

#[derive(Resource, Clone, Debug, Default)]
struct UiState {
  show_wireframe: bool,
}

fn toggle_wireframe(
  input: Res<Input<KeyCode>>,
  query: Query<Entity, With<Handle<Mesh>>>,
  mut commands: Commands,
  mut state: ResMut<UiState>,
) {
  if input.just_pressed(KeyCode::R) {
    // Update flag.
    let show_wireframe = !state.show_wireframe;
    state.show_wireframe = show_wireframe;

    for entity in query.iter() {
      let mut entity = commands.entity(entity);
      if show_wireframe {
        entity.insert(Wireframe);
      } else {
        entity.remove::<Wireframe>();
      }
    }
  }
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer {
  update: Timer,
  time: Stopwatch,
  speed: f32,
}

impl CycleTimer {
  pub fn new(duration: Duration, speed: f32) -> Self {
    Self {
      update: Timer::new(duration, TimerMode::Repeating),
      time: Stopwatch::new(),
      speed,
    }
  }

  pub fn tick(&mut self, delta: Duration) {
    if !self.paused() {
      self.update.tick(delta);
      self.time.tick(delta.mul_f32(self.speed));
    }
  }

  pub fn paused(&self) -> bool {
    self.time.paused()
  }

  pub fn toggle_pause(&mut self) {
    if self.time.paused() {
      self.time.unpause();
    } else {
      self.time.pause();
    }
  }

  pub fn time(&self) -> f32 {
    self.time.elapsed().as_millis() as f32 / 2000.0
  }

  pub fn update(&self) -> bool {
    self.update.finished()
  }

  pub fn update_speed(&mut self, delta: f32) {
    self.speed += delta;
    if self.speed < SPEED_MIN {
      self.speed = SPEED_MIN;
    }
    if self.speed > SPEED_MAX {
      self.speed = SPEED_MAX;
    }
  }
}

fn timer_control(input: Res<Input<KeyCode>>, mut timer: ResMut<CycleTimer>) {
  if input.just_pressed(KeyCode::P) {
    timer.toggle_pause();
  }
  if input.pressed(KeyCode::NumpadAdd) {
    timer.update_speed(SPEED_DELTA);
    eprintln!("Increase speed: {}", timer.speed);
  }
  if input.pressed(KeyCode::NumpadSubtract) {
    timer.update_speed(-SPEED_DELTA);
    eprintln!("Decrease speed: {}", timer.speed);
  }
}

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
  mut atmosphere: AtmosphereMut<Nishita>,
  mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
  mut timer: ResMut<CycleTimer>,
  time: Res<Time>,
) {
  // Do nothing if timer is paused.
  if timer.paused() {
    return;
  }

  timer.tick(time.delta());

  if timer.update() {
    let mut pos = atmosphere.sun_position;
    let t = (timer.time() + 3.0) * 0.1;
    pos.y = t.sin();
    pos.z = t.cos();
    atmosphere.sun_position = pos;

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
      light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
      directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
    }
  }
}

#[derive(Bundle, Default)]
struct ShipBundle {
  ship: Ship,
  name: Name,
  #[bundle]
  scene: SceneBundle,
}

#[derive(Component, Default, Clone)]
struct Ship {
  water_line: f32,
  front: Vec3,
  back_left: Vec3,
  back_right: Vec3,
}

impl Ship {
  fn new(water_line: f32, front: f32, back: f32, left: f32, right: f32) -> Self {
    Self {
      water_line,
      front: Vec3::new(0.0, 0.0, front),
      back_left: Vec3::new(left, 0.0, back),
      back_right: Vec3::new(right, 0.0, back),
    }
  }

  fn update(
    &self,
    water: &WaterParam,
    pos: Vec3,
    transform: &mut Transform,
    #[cfg(feature = "debug")]
    lines: &mut DebugLines
  ) {
    let (yaw, _pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
    let global = Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw));

    // Get the wave position at the front, back_left and back_right.
    let mut front = water.wave_point(global.transform_point(self.front));
    let left = water.wave_point(global.transform_point(self.back_left));
    let right = water.wave_point(global.transform_point(self.back_right));
    let normal = (left - front).cross(right - front).normalize();

    // Debug lines.
    #[cfg(feature = "debug")]
    {
      lines.line(front, front + normal, 0.0);
      lines.line_colored(front, right, 0.0, Color::RED);
      lines.line(right, left, 0.0);
      lines.line_colored(left, front, 0.0, Color::GREEN);
    }

    front.y += self.water_line - 0.2;
    transform.look_at(front, normal);

    transform.translation.y = ((front.y + left.y + right.y) / 3.0) + self.water_line;
  }
}

fn update_ships(
  water: WaterParam,
  mut ships: Query<(&Ship, &mut Transform, &GlobalTransform)>,
  #[cfg(feature = "debug")]
  mut lines: ResMut<DebugLines>
) {
  for (ship, mut transform, global) in ships.iter_mut() {
    let pos = global.translation();
    #[cfg(not(feature = "debug"))]
    ship.update(&water, pos, &mut transform);
    #[cfg(feature = "debug")]
    ship.update(&water, pos, &mut transform, &mut lines);
  }
}

/// set up a simple 3D scene
fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // "Sun"
  commands
    .spawn(DirectionalLightBundle {
      directional_light: DirectionalLight {
        shadows_enabled: true,
        ..default()
      },
      ..default()
    })
    .insert(Sun); // Marks the light as Sun

  // Terrain material.
  let material = materials.add(StandardMaterial {
    base_color: Color::rgba_u8(177, 168, 132, 255),
    perceptual_roughness: 0.6,
    metallic: 0.6,
    reflectance: 0.8,
    ..default()
  });

  // Spawn simple terrain plane.
  commands
    .spawn((
      Name::new(format!("Terrain")),
      MaterialMeshBundle {
        mesh: meshes.add(
          shape::Plane {
            size: 256.0 * 6.0,
            ..default()
          }
          .into(),
        ),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, -5.0, 0.0),
        ..default()
      },
      NotShadowCaster,
    ));
  // Spawn fake island.
  commands
    .spawn((
      Name::new(format!("Fake island")),
      MaterialMeshBundle {
        mesh: meshes.add(
          shape::Icosphere {
            radius: 2.0,
            ..default()
          }
          .try_into().expect("Icosphere"),
        ),
        material: material.clone(),
        transform: Transform::from_xyz(-30.0, -10.0, -30.0)
          .with_scale(Vec3::new(30.0, 6.5, 30.0)),
        ..default()
      },
      NotShadowCaster,
    ));

  // camera
  commands.spawn((
    Camera3dBundle {
      ..default()
    },
    AtmosphereCamera::default(),
    PanOrbitCamera {
      focus: Vec3::new(25.0, WATER_HEIGHT + 5.0, -61.0),
      radius: 4.0,
      alpha: 3.14,
      beta: 0.0,
      ..default()
    },
    // This will write the depth buffer to a texture that you can use in the main pass
    DepthPrepass,
  ));

  // Spawn ships.
  let scene = asset_server.load("models/Kenney_pirate/ship_dark.gltf#Scene0");
  let ship = Ship::new(-0.400, -3.8, 2.5, -1.4, 1.4);

  // "Randomly" place the ships.
  for x in 1..18 {
    let f = (x as f32) * 1.20;
    let f2 = ((x % 6) as f32) * -10.90;
    commands.spawn(ShipBundle {
      ship: ship.clone(),
      name: Name::new(format!("Ship {x}")),
      scene: SceneBundle {
        scene: scene.clone(),
        transform: Transform::from_xyz(-20.0 + (f * 7.8), 0.0, 20.0 + f2)
          .with_rotation(Quat::from_rotation_y(f)),
        ..default()
      },
      ..default()
    });
  }

  info!("Move camera around by using WASD for lateral movement");
  info!("Use Left Shift and Spacebar for vertical movement");
  info!("Use the mouse to look around");
  info!("Press Esc to hide or show the mouse cursor");
}
