//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

#[cfg(not(feature = "atmosphere"))]
use bevy::core_pipeline::Skybox;

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::mesh::*;
use bevy::{
  app::AppExit,
  prelude::*,
  render::{render_resource::TextureFormat},
};
#[cfg(feature = "atmosphere")]
use bevy::{time::Stopwatch, utils::Duration};

#[cfg(feature = "atmosphere")]
use bevy_atmosphere::prelude::*;
#[cfg(feature = "panorbit")]
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
#[cfg(feature = "spectator")]
use bevy_spectator::*;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use bevy_water::*;

pub const WATER_HEIGHT: f32 = 1.0;
#[cfg(feature = "atmosphere")]
pub const SPEED_MIN: f32 = 0.05;
#[cfg(feature = "atmosphere")]
pub const SPEED_DELTA: f32 = 0.01;
#[cfg(feature = "atmosphere")]
pub const SPEED_MAX: f32 = 1.0;

pub fn pirates_app(title: &str) -> App {
  let mut app = App::new();
  app
    // Tell the asset server to watch for asset changes on disk:
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: title.to_string(),
            resolution: (1200, 600).into(),
            ..Default::default()
          }),
          ..default()
        })
        .set(AssetPlugin::default()),
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
  app
    .insert_resource(bevy::light::DirectionalLightShadowMap { size: 4 * 1024 })
    // Water
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    })
    .add_plugins((WaterPlugin, ImageUtilsPlugin))
    // Ship Physics.
    .add_systems(Update, update_ships)
    // Quit on `Q` press.
    .add_systems(Update, handle_quit)
    // Wireframe
    .add_plugins(WireframePlugin::default())
    .init_resource::<UiState>()
    .add_systems(Update, toggle_wireframe);

  // Atmosphere + daylight cycle.
  #[cfg(feature = "atmosphere")]
  app
    .insert_resource(AtmosphereModel::new(Nishita {
      sun_position: Vec3::new(0.0, 1.0, 1.0),
      ..default()
    }))
    .add_plugins(AtmospherePlugin)
    .insert_resource(CycleTimer::new(Duration::from_millis(1000), 0.2))
    .add_systems(Update, timer_control)
    .add_systems(Update, daylight_cycle);

  app
}

#[allow(dead_code)]
pub fn main() {
  let mut app = pirates_app("Pirates");

  // Setup
  app.add_systems(Startup, (setup_ocean, setup_orb, setup_camera, setup_ships));

  app.run();
}

pub fn handle_quit(input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
  if input.pressed(KeyCode::KeyQ) {
    exit.write(AppExit::Success);
  }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct UiState {
  show_wireframe: bool,
}

pub fn toggle_wireframe(
  input: Res<ButtonInput<KeyCode>>,
  query: Query<Entity, With<Mesh3d>>,
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
pub struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
#[cfg(feature = "atmosphere")]
pub struct CycleTimer {
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
    if !self.is_paused() {
      self.update.tick(delta);
      self.time.tick(delta.mul_f32(self.speed));
    }
  }

  pub fn is_paused(&self) -> bool {
    self.time.is_paused()
  }

  pub fn toggle_pause(&mut self) {
    if self.time.is_paused() {
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
pub fn timer_control(input: Res<ButtonInput<KeyCode>>, mut timer: ResMut<CycleTimer>) {
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
pub fn daylight_cycle(
  mut atmosphere: AtmosphereMut<Nishita>,
  mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
  mut timer: ResMut<CycleTimer>,
  time: Res<Time>,
) {
  // Do nothing if timer is paused.
  if timer.is_paused() {
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

#[derive(Component, Default, Clone)]
#[require(Transform, Visibility)]
pub struct Ship {
  water_line: f32,
  front: Vec3,
  back_left: Vec3,
  back_right: Vec3,
}

impl Ship {
  pub fn new(water_line: f32, front: f32, back: f32, left: f32, right: f32) -> Self {
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
    #[cfg(feature = "debug")] lines: &mut DebugLines,
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

pub fn update_ships(
  water: WaterParam,
  mut ships: Query<(&Ship, &mut Transform, &GlobalTransform)>,
  #[cfg(feature = "debug")] mut lines: ResMut<DebugLines>,
) {
  for (ship, mut transform, global) in ships.iter_mut() {
    let pos = global.translation();
    #[cfg(not(feature = "debug"))]
    ship.update(&water, pos, &mut transform);
    #[cfg(feature = "debug")]
    ship.update(&water, pos, &mut transform, &mut lines);
  }
}

pub fn scale_uvs(mesh: &mut Mesh, scale: f32) {
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

/// set up a simple ocean scene.
pub fn setup_ocean(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // "Sun"
  commands
    .spawn((
      DirectionalLight {
        illuminance: 11127.65,
        shadows_enabled: true,
        ..default()
      },
      Transform::from_rotation(Quat::from_rotation_x(-0.340)),
    ))
    .insert(Sun); // Marks the light as Sun

  // Prepare textures.
  let base_color_texture = Some(asset_server.load("textures/coast_sand_01_1k/diff.jpg"));
  let metallic_roughness_texture = Some(ImageReformat::reformat(
    &mut commands,
    &asset_server,
    "textures/coast_sand_01_1k/rough.jpg",
    TextureFormat::Rgba8Unorm,
  ));
  let normal_map_texture = Some(ImageReformat::reformat(
    &mut commands,
    &asset_server,
    "textures/coast_sand_01_1k/normal.jpg",
    TextureFormat::Rgba8Unorm,
  ));
  ImageReformat::uv_repeat(
    &mut commands,
    &asset_server,
    "textures/coast_sand_01_1k/diff.jpg",
  );
  ImageReformat::uv_repeat(
    &mut commands,
    &asset_server,
    "textures/coast_sand_01_1k/rough.jpg",
  );
  ImageReformat::uv_repeat(
    &mut commands,
    &asset_server,
    "textures/coast_sand_01_1k/normal.jpg",
  );

  // Coast sand material.
  let sandy = MeshMaterial3d(materials.add(StandardMaterial {
    perceptual_roughness: 1.0,
    metallic: 0.0,
    reflectance: 0.5,
    base_color_texture,
    metallic_roughness_texture,
    normal_map_texture,
    cull_mode: None,
    double_sided: true,
    ..default()
  }));

  let floor_mesh = Mesh3d({
    let mut mesh = PlaneMeshBuilder::from_length(256.0 * 6.0)
      .subdivisions(25)
      .build();
    mesh.generate_tangents().expect("tangents");
    scale_uvs(&mut mesh, 50.0);
    meshes.add(mesh)
  });
  commands.spawn((
    Name::new(format!("Sea floor")),
    floor_mesh.clone(),
    sandy.clone(),
    Transform::from_xyz(0.0, -5.0, 0.0),
  ));

  let island_mesh = Mesh3d({
    let mut mesh = Sphere::new(2.0)
      .mesh()
      .kind(SphereKind::Uv {
        sectors: 90,
        stacks: 60,
      })
      .build();
    mesh.generate_tangents().expect("tangents");
    scale_uvs(&mut mesh, 20.0);
    meshes.add(mesh)
  });
  commands.spawn((
    Name::new(format!("Sandy island")),
    island_mesh.clone(),
    sandy.clone(),
    Transform::from_xyz(-30.0, -10.0, -30.0).with_scale(Vec3::new(30.0, 6.5, 30.0)),
  ));
}

/// Create a simple Orb.
pub fn setup_orb(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let orb_mesh = Mesh3d({
    let mut mesh = Sphere::new(1.0)
      .mesh()
      .kind(SphereKind::Uv {
        sectors: 90,
        stacks: 60,
      })
      .build();
    mesh.generate_tangents().expect("tangents");
    meshes.add(mesh)
  });
  commands.spawn((
    Name::new(format!("Orb")),
    orb_mesh.clone(),
    MeshMaterial3d(materials.add(Color::srgba(0.1, 0.2, 0.4, 1.0))),
    Transform::from_xyz(-30.0, 10.0, -30.0),
  ));
}

/// Create a simple 3D camera
pub fn make_camera<'a>(
  commands: &'a mut Commands,
  asset_server: &AssetServer,
) -> EntityCommands<'a> {
  // camera
  let mut cam = commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(-20.0, WATER_HEIGHT + 5.0, 20.0)
      .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
    EnvironmentMapLight {
      diffuse_map: asset_server.load("environment_maps/table_mountain_2_puresky_4k_diffuse.ktx2"),
      specular_map: asset_server.load("environment_maps/table_mountain_2_puresky_4k_specular.ktx2"),
      intensity: 1.0,
      ..default()
    },
    DistanceFog {
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
    focus: Vec3::new(26.0, WATER_HEIGHT + 5.0, -11.0),
    radius: Some(60.0),
    yaw: Some(-std::f32::consts::FRAC_PI_2),
    pitch: Some(0.0),
    ..default()
  });

  #[cfg(feature = "atmosphere")]
  cam.insert(AtmosphereCamera::default());

  #[cfg(not(feature = "atmosphere"))]
  {
    cam.insert(Skybox {
      image: asset_server.load("environment_maps/table_mountain_2_puresky_4k_cubemap.ktx2"),
      brightness: 2000.0,
      ..default()
    });
  }

  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }
  // This is just to keep the compiler happy when not using `depth_prepass` feature.
  cam.insert(Name::new("Camera"));

  info!("Move camera around by using WASD for lateral movement");
  info!("Use Left Shift and Spacebar for vertical movement");
  info!("Use the mouse to look around");
  info!("Press Esc to hide or show the mouse cursor");

  cam
}

/// set up a simple 3D camera
pub fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
  make_camera(&mut commands, &asset_server);
}

/// Spawn some dutch ships.
pub fn setup_ships(mut commands: Commands, asset_server: Res<AssetServer>) {
  // Spawn ships.
  let scene =
    SceneRoot(asset_server.load("models/dutch_ship_medium_1k/dutch_ship_medium_1k.gltf#Scene0"));
  let ship = Ship::new(-0.400, -8.0, 9.0, -2.0, 2.0);

  // "Randomly" place the ships.
  for x in 1..10 {
    let f = (x as f32) * 2.40;
    let f2 = ((x % 6) as f32) * -20.90;
    commands
      .spawn((
        ship.clone(),
        Name::new(format!("Dutch Ship {x}")),
        Transform::from_xyz(-10.0 + (f * 7.8), 0.0, 30.0 + f2)
          .with_rotation(Quat::from_rotation_y(f)),
      ))
      .with_children(|parent| {
        parent.spawn((
          scene.clone(),
          // Rotate ship model to line up with rotation axis.
          Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ));
      });
  }
}
