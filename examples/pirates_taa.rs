//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

#[cfg(not(feature = "atmosphere"))]
use bevy::core_pipeline::Skybox;

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::{
  app::AppExit, asset::ChangeWatcher, prelude::*, utils::Duration,
  core_pipeline::{
    experimental::taa::{
      TemporalAntiAliasBundle, TemporalAntiAliasPlugin,
    },
  },
  render::{
    mesh::VertexAttributeValues,
    render_resource::TextureFormat,
  },
};
#[cfg(feature = "atmosphere")]
use bevy::{
  time::Stopwatch,
};

#[cfg(feature = "atmosphere")]
use bevy_atmosphere::prelude::*;
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

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
      }).set(AssetPlugin {
      // Tell the asset server to watch for asset changes on disk:
      watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
      ..default()
    }))
    .add_plugins(TemporalAntiAliasPlugin);

  #[cfg(feature = "debug")]
  app.add_plugins(DebugLinesPlugin::with_depth_test(true))
    .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

  // Simple pan/orbit camera.
  app.add_plugins(PanOrbitCameraPlugin);

  // Improve shadows.
  app.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4 * 1024 })
    // Water
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    })
    .add_plugins((WaterPlugin, ImageUtilsPlugin))
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

  app.run();
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
  spatial: SpatialBundle,
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
      lines.line(transform.translation, transform.translation + normal, 0.0);
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

fn scale_uvs(mesh: &mut Mesh, scale: f32) {
  match mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
    Some(VertexAttributeValues::Float32x2(uvs)) => {
      for [x, y] in uvs.iter_mut() {
        *x *= scale;
        *y *= scale;
      }
    }
    Some(_) => {
      panic!("Unexpected UV format");
    }
    _ => {
      panic!("Mesh doesn't have UVS");
    }
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

  // Prepare textures.
  let base_color_texture = Some(asset_server.load("textures/coast_sand_01_1k/diff.jpg"));
  let metallic_roughness_texture =
    Some(ImageReformat::reformat(&mut commands, &asset_server, "textures/coast_sand_01_1k/rough.jpg", TextureFormat::Rgba8Unorm));
  let normal_map_texture =
    Some(ImageReformat::reformat(&mut commands, &asset_server, "textures/coast_sand_01_1k/normal.jpg", TextureFormat::Rgba8Unorm));
  ImageReformat::uv_repeat(&mut commands, &asset_server, "textures/coast_sand_01_1k/diff.jpg");
  ImageReformat::uv_repeat(&mut commands, &asset_server, "textures/coast_sand_01_1k/rough.jpg");
  ImageReformat::uv_repeat(&mut commands, &asset_server, "textures/coast_sand_01_1k/normal.jpg");

  // Coast sand material.
  let sandy = materials.add(StandardMaterial {
    perceptual_roughness: 1.0,
    metallic: 0.0,
    reflectance: 0.5,
    base_color_texture,
    metallic_roughness_texture,
    normal_map_texture,
    cull_mode: None,
    double_sided: true,
    ..default()
  });

  let floor_mesh = {
    let mut mesh = Mesh::from(shape::Plane {
      size: 256.0 * 6.0,
      subdivisions: 25,
    });
    mesh.generate_tangents().expect("tangents");
    scale_uvs(&mut mesh, 50.0);
    meshes.add(mesh)
  };
  commands
    .spawn((
      Name::new(format!("Sea floor")),
      MaterialMeshBundle {
        mesh: floor_mesh.clone(),
        material: sandy.clone(),
        transform: Transform::from_xyz(0.0, -5.0, 0.0),
        ..default()
      },
    ));

  let island_mesh = {
    let mut mesh = Mesh::from(shape::UVSphere {
            radius: 2.0,
            sectors: 90,
            stacks: 60,
    });
    mesh.generate_tangents().expect("tangents");
    scale_uvs(&mut mesh, 20.0);
    meshes.add(mesh)
  };
  commands
    .spawn((
      Name::new(format!("Sandy island")),
      MaterialMeshBundle {
        mesh: island_mesh.clone(),
        material: sandy.clone(),
        transform: Transform::from_xyz(-30.0, -10.0, -30.0)
          .with_scale(Vec3::new(30.0, 6.5, 30.0)),
        ..default()
      },
    ));

  // Skybox cubemap
  #[cfg(not(feature = "atmosphere"))]
  let skybox_handle = ImageReformat::cubemap(&mut commands, &asset_server, SKYBOX_NAME);

  // camera
  let mut cam = commands.spawn((
    Camera3dBundle {
      ..default()
    },
    FogSettings {
        color: Color::rgba(0.1, 0.2, 0.4, 1.0),
        //directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
        //directional_light_exponent: 30.0,
        falloff: FogFalloff::from_visibility_colors(
            400.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
            Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
            Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
        ),
        ..default()
    },
    TemporalAntiAliasBundle::default(),
  ));

  cam.insert(PanOrbitCamera {
    focus: Vec3::new(26.0, WATER_HEIGHT + 5.0, -11.0),
    radius: Some(60.0),
    alpha: Some(-std::f32::consts::FRAC_PI_2),
    beta: Some(0.0),
    ..default()
  });

  #[cfg(feature = "atmosphere")]
  cam.insert(AtmosphereCamera::default());

  #[cfg(not(feature = "atmosphere"))]
  {
    cam.insert(Skybox(skybox_handle));
  }

  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }
  // This is just to keep the compiler happy when not using `depth_prepass` feature.
  cam.insert(Name::new("Camera"));

  // Spawn ships.
  let scene = asset_server.load("models/dutch_ship_medium_1k/dutch_ship_medium_1k.gltf#Scene0");
  let ship = Ship::new(-0.400, -8.0, 9.0, -2.0, 2.0);

  // "Randomly" place the ships.
  for x in 1..10 {
    let f = (x as f32) * 2.40;
    let f2 = ((x % 6) as f32) * -20.90;
    commands.spawn(ShipBundle {
      ship: ship.clone(),
      name: Name::new(format!("Dutch Ship {x}")),
      spatial: SpatialBundle {
        transform: Transform::from_xyz(-10.0 + (f * 7.8), 0.0, 30.0 + f2)
          .with_rotation(Quat::from_rotation_y(f)),
        ..default()
      },
      ..default()
    })
    .with_children(|parent| {
      parent.spawn(SceneBundle {
        scene: scene.clone(),
        // Rotate ship model to line up with rotation axis.
        transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ..default()
      });
    });
  }

  info!("Move camera around by using WASD for lateral movement");
  info!("Use Left Shift and Spacebar for vertical movement");
  info!("Use the mouse to look around");
  info!("Press Esc to hide or show the mouse cursor");
}
