//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

#[cfg(not(feature = "atmosphere"))]
use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
    },
};

use bevy::pbr::NotShadowCaster;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::render::mesh::*;
use bevy::{app::AppExit, prelude::*};
#[cfg(feature = "atmosphere")]
use bevy::{
  time::Stopwatch,
  utils::Duration,
};

#[cfg(feature = "atmosphere")]
use bevy_atmosphere::prelude::*;
#[cfg(feature = "panorbit")]
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};
#[cfg(feature = "spectator")]
use bevy_spectator::*;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use bevy_water::*;

#[cfg(not(feature = "atmosphere"))]
const SKYBOX_NAME: &str = "textures/table_mountain_2_puresky_4k_cubemap.jpg";

const WATER_HEIGHT: f32 = 1.0;
#[cfg(feature = "atmosphere")]
const SPEED_MIN: f32 = 0.05;
#[cfg(feature = "atmosphere")]
const SPEED_DELTA: f32 = 0.01;
#[cfg(feature = "atmosphere")]
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
      }).set(AssetPlugin::default())
    );

  #[cfg(feature = "debug")]
  app.add_plugins(DebugLinesPlugin::with_depth_test(true));

  #[cfg(feature = "inspector")]
  app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

  // Simple movement for this example
  #[cfg(feature = "spectator")]
  app.add_plugins(SpectatorPlugin);

  // Simple pan/orbit camera.
  #[cfg(feature = "panorbit")]
  app.add_plugins(PanOrbitCameraPlugin);

  // Improve shadows.
  app.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4 * 1024 })
    // Water
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    })
    .add_plugins(WaterPlugin)
    // Ship Physics.
    .add_systems(Update, update_ships)
    // Setup
    .add_systems(Startup, setup)
    .add_systems(Update, handle_quit)
    // Wireframe
    .add_plugins(WireframePlugin)
    .init_resource::<UiState>()
    .add_systems(Update, toggle_wireframe);

  // Atmosphere + daylight cycle.
  #[cfg(feature = "atmosphere")]
  app.insert_resource(AtmosphereModel::new(Nishita {
      sun_position: Vec3::new(0.0, 1.0, 1.0),
      ..default()
    }))
    .add_plugins(AtmospherePlugin)
    .insert_resource(CycleTimer::new(
      Duration::from_millis(1000),
      0.2,
    ))
    .add_systems(Update, timer_control)
    .add_systems(Update, daylight_cycle);

  #[cfg(not(feature = "atmosphere"))]
  app.add_systems(
      Update,
      asset_loaded,
  );

  app.run();
}

fn handle_quit(input: Res<ButtonInput<KeyCode>>, mut _exit: EventWriter<AppExit>) {
  if input.pressed(KeyCode::KeyQ) {
    //_exit.send(AppExit::Success);
  }
}

#[derive(Resource, Clone, Debug, Default)]
struct UiState {
  show_wireframe: bool,
}

fn toggle_wireframe(
  input: Res<ButtonInput<KeyCode>>,
  query: Query<Entity, With<Handle<Mesh>>>,
  mut commands: Commands,
  mut state: ResMut<UiState>,
) {
  if input.just_pressed(KeyCode::KeyR) {
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
#[cfg(feature = "atmosphere")]
struct CycleTimer {
  update: Timer,
  time: Stopwatch,
  speed: f32,
}

#[cfg(feature = "atmosphere")]
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

#[cfg(feature = "atmosphere")]
fn timer_control(input: Res<ButtonInput<KeyCode>>, mut timer: ResMut<CycleTimer>) {
  if input.just_pressed(KeyCode::KeyP) {
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
#[cfg(feature = "atmosphere")]
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

#[derive(Resource)]
#[cfg(not(feature = "atmosphere"))]
struct Cubemap {
    is_loaded: bool,
    name: String,
    image_handle: Handle<Image>,
}

#[cfg(not(feature = "atmosphere"))]
fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
) {
    if !cubemap.is_loaded
        && asset_server.get_load_state(&cubemap.image_handle) == Some(LoadState::Loaded)
    {
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            info!("Reinterpret 2D image {} into Cubemap", cubemap.name);
            image.reinterpret_stacked_2d_as_array(
                image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        cubemap.is_loaded = true;
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
        illuminance: 11127.65,
        shadows_enabled: true,
        ..default()
      },
      transform: Transform::from_rotation(Quat::from_rotation_x(-0.340)),
      ..default()
    })
    .insert(Sun); // Marks the light as Sun

  // Terrain material.
  let material = materials.add(StandardMaterial {
    base_color: Color::srgba_u8(177, 168, 132, 255),
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
          PlaneMeshBuilder::from_length(256.0 * 6.0)
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
          Sphere::new(2.0)
        ),
        material: material.clone(),
        transform: Transform::from_xyz(-30.0, -10.0, -30.0)
          .with_scale(Vec3::new(30.0, 6.5, 30.0)),
        ..default()
      },
      NotShadowCaster,
    ));

  // Skybox cubemap
  #[cfg(not(feature = "atmosphere"))]
  let skybox_handle ={
    let handle = asset_server.load(SKYBOX_NAME);
    commands.insert_resource(Cubemap {
        is_loaded: false,
        name: SKYBOX_NAME.to_string(),
        image_handle: handle.clone(),
    });
    handle
  };

  // camera
  let mut cam = commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(-20.0, WATER_HEIGHT + 5.0, 20.0)
        .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
      ..default()
    },
    FogSettings {
        color: Color::srgba(0.1, 0.2, 0.4, 1.0),
        //directional_light_color: Color::srgba(1.0, 0.95, 0.75, 0.5),
        //directional_light_exponent: 30.0,
        falloff: FogFalloff::from_visibility_colors(
            400.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
            Color::srgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
            Color::srgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
        ),
        ..default()
    },
  ));

  #[cfg(feature = "spectator")]
  cam.insert(Spectator);

  #[cfg(feature = "panorbit")]
  cam.insert(PanOrbitCamera {
    focus: Vec3::new(25.0, WATER_HEIGHT + 5.0, -61.0),
    radius: Some(4.0),
    yaw: Some(3.14),
    pitch: Some(0.0),
    ..default()
  });

  #[cfg(feature = "atmosphere")]
  cam.insert(AtmosphereCamera::default());

  #[cfg(not(feature = "atmosphere"))]
  {
    cam.insert(Skybox {
      image: skybox_handle,
      brightness: 2000.0,
    });
  }

  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }
  // This is just to keep the compiler happy when not using `depth_prepass` feature.
  cam.insert(Name::new("Camera"));

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
