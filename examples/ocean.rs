//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::time::Stopwatch;
use bevy::utils::Duration;
use bevy::{app::AppExit, prelude::*};

use bevy_atmosphere::prelude::*;
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};

use bevy_water::*;

const WATER_HEIGHT: f32 = 20.0;
const SPEED_MIN: f32 = 0.05;
const SPEED_DELTA: f32 = 0.01;
const SPEED_MAX: f32 = 1.0;

fn main() {
  App::new()
    .insert_resource(Msaa { samples: 4 })
    // Tell the asset server to watch for asset changes on disk:
    .add_plugins(DefaultPlugins.set(AssetPlugin {
      watch_for_changes: true,
      ..default()
    }))
    // Atmosphere + daylight cycle.
    .insert_resource(AtmosphereModel::new(Nishita {
      sun_position: Vec3::new(0.0, 1.0, 1.0),
      ..default()
    }))
    .add_plugin(AtmospherePlugin)
    .insert_resource(CycleTimer::new(
      // Update our atmosphere every 50ms (in a real game, this would be much slower, but for the sake of an example we use a faster update)
      Duration::from_millis(50),
      0.6,
    ))
    .add_system(timer_control)
    .add_system(daylight_cycle)
    // Camera
    .add_plugin(NoCameraPlayerPlugin)
    .insert_resource(MovementSettings {
      sensitivity: 0.00015, // default: 0.00012
      speed: 12.0,          // default: 12.0
    })
    // Water
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
    })
    .add_plugin(WaterPlugin)
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
    let t = timer.time();
    pos.y = t.sin();
    pos.z = t.cos();
    atmosphere.sun_position = pos;

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
      light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
      directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
    }
  }
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
    material: materials.add(Color::rgb(0.5, 0.3, 0.3).into()),
    transform: Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
    ..default()
  });
  // cube
  commands.spawn(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    transform: Transform::from_xyz(0.0, WATER_HEIGHT, 0.0),
    ..default()
  });

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

  // camera
  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(-20.0, WATER_HEIGHT + 5.0, 20.0)
        .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
      ..default()
    },
    AtmosphereCamera::default(),
    FlyCam,
  ));

  info!("Move camera around by using WASD for lateral movement");
  info!("Use Left Shift and Spacebar for vertical movement");
  info!("Use the mouse to look around");
  info!("Press Esc to hide or show the mouse cursor");
}
