//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

#[cfg(not(feature = "atmosphere"))]
use bevy::core_pipeline::Skybox;

#[cfg(all(
  not(feature = "atmosphere"),
  not(target_arch = "wasm32"),
  not(feature = "ssr")
))]
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
#[cfg(feature = "debug")]
use bevy::color::palettes::css::*;
use bevy::mesh::*;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
#[cfg(not(feature = "atmosphere"))]
use bevy::pbr::ScreenSpaceAmbientOcclusion;
#[cfg(feature = "ssr")]
use bevy::pbr::ScreenSpaceReflections;
#[cfg(feature = "atmosphere")]
use bevy::{
  anti_alias::fxaa::Fxaa,
  core_pipeline::tonemapping::Tonemapping,
  light::{AtmosphereEnvironmentMapLight, VolumetricFog},
  pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings},
  post_process::bloom::Bloom,
};
use bevy::{
  app::AppExit,
  camera::Exposure,
  input::{common_conditions, keyboard::KeyCode},
  light::{light_consts::lux, FogVolume, VolumetricLight},
  pbr::ScatteringMedium,
  prelude::*,
  render::{render_resource::TextureFormat, view::Hdr},
};

use bevy_water::*;

#[derive(Resource, Default)]
struct GameState {
  paused: bool,
}

pub const WATER_HEIGHT: f32 = 1.0;

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
    )
    .add_plugins(FreeCameraPlugin);

  #[cfg(feature = "inspector")]
  app.add_plugins((
    bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
    bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
  ));

  #[cfg(feature = "ssr")]
  app.insert_resource(bevy::pbr::DefaultOpaqueRendererMethod::deferred());

  app
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(GameState::default())
    .insert_resource(GlobalAmbientLight::NONE)
    // Improve shadows.
    // .insert_resource(bevy::light::DirectionalLightShadowMap { size: 4 * 1024 })
    // Water
    .insert_resource(WaterSettings {
      height: WATER_HEIGHT,
      ..default()
    })
    .add_plugins((WaterPlugin, ImageUtilsPlugin))
    // Ship Physics.
    .add_systems(Update, update_ships)
    // Wireframe
    .add_plugins(WireframePlugin::default())
    .add_systems(Update, common_controls)
    .add_systems(
      Update,
      toggle_wireframe.run_if(common_conditions::input_just_pressed(KeyCode::KeyR)),
    );

  // Atmosphere + daylight cycle.
  #[cfg(feature = "atmosphere")]
  app
    .add_systems(Startup, print_controls)
    .add_systems(Update, (daylight_cycle, atmosphere_controls));

  app
}

#[cfg(feature = "atmosphere")]
fn print_controls() {
  println!("Atmosphere Example Controls:");
  println!("    1          - Switch to lookup texture rendering method");
  println!("    2          - Switch to raymarched rendering method");
  println!("    Enter      - Pause/Resume sun motion");
  println!("    Up/Down    - Increase/Decrease exposure");
}

fn common_controls(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut exit: MessageWriter<AppExit>,
  mut game_state: ResMut<GameState>,
  mut camera_exposure: Query<&mut Exposure, With<Camera3d>>,
  time: Res<Time>,
) {
  if keyboard_input.just_pressed(KeyCode::Enter) {
    game_state.paused = !game_state.paused;
  }

  if keyboard_input.pressed(KeyCode::ArrowUp) {
    for mut exposure in &mut camera_exposure {
      exposure.ev100 -= time.delta_secs() * 2.0;
    }
  }

  if keyboard_input.pressed(KeyCode::ArrowDown) {
    for mut exposure in &mut camera_exposure {
      exposure.ev100 += time.delta_secs() * 2.0;
    }
  }

  if keyboard_input.pressed(KeyCode::Escape) {
    exit.write(AppExit::Success);
  }
}

#[cfg(feature = "atmosphere")]
fn atmosphere_controls(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut atmosphere_settings: Query<&mut AtmosphereSettings>,
) {
  if keyboard_input.just_pressed(KeyCode::Digit1) {
    for mut settings in &mut atmosphere_settings {
      settings.rendering_method = AtmosphereMode::LookupTexture;
      println!("Switched to lookup texture rendering method");
    }
  }

  if keyboard_input.just_pressed(KeyCode::Digit2) {
    for mut settings in &mut atmosphere_settings {
      settings.rendering_method = AtmosphereMode::Raymarched;
      println!("Switched to raymarched rendering method");
    }
  }
}

#[allow(dead_code)]
pub fn main() {
  let mut app = pirates_app("Pirates");

  // Setup
  app.add_systems(Startup, (setup_ocean, setup_orb, setup_camera, setup_ships));

  app.run();
}

pub fn toggle_wireframe(
  mut show_wireframe: Local<bool>,
  query: Query<Entity, With<Mesh3d>>,
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

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
pub struct Sun;

#[cfg(feature = "atmosphere")]
fn daylight_cycle(
  mut suns: Query<&mut Transform, With<Sun>>,
  time: Res<Time>,
  sun_motion_state: Res<GameState>,
) {
  use std::f32::consts::PI;
  // Only rotate the sun if motion is not paused
  if !sun_motion_state.paused {
    suns
      .iter_mut()
      .for_each(|mut tf| tf.rotate_x(-time.delta_secs() * PI / 10.0));
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
    #[cfg(feature = "debug")] gizmos: &mut Gizmos,
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
      gizmos.line(front, front + normal, WHITE);
      gizmos.line(front, right, RED);
      gizmos.line(right, left, WHITE);
      gizmos.line(left, front, GREEN);
      gizmos.line(transform.translation, transform.translation + normal, WHITE);
    }

    front.y += self.water_line - 0.2;
    transform.look_at(front, normal);

    transform.translation.y = ((front.y + left.y + right.y) / 3.0) + self.water_line;
  }
}

pub fn update_ships(
  water: WaterParam,
  mut ships: Query<(&Ship, &mut Transform, &GlobalTransform)>,
  #[cfg(feature = "debug")] mut gizmos: Gizmos,
) {
  for (ship, mut transform, global) in ships.iter_mut() {
    let pos = global.translation();
    #[cfg(not(feature = "debug"))]
    ship.update(&water, pos, &mut transform);
    #[cfg(feature = "debug")]
    ship.update(&water, pos, &mut transform, &mut gizmos);
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
  let sphere_mesh = meshes.add(Mesh::from(Sphere { radius: 1.0 }));

  // light probe spheres
  commands.spawn((
    Mesh3d(sphere_mesh.clone()),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::WHITE,
      metallic: 1.0,
      perceptual_roughness: 0.0,
      ..default()
    })),
    Transform::from_xyz(-1.0, 1.0, -1.0),
    Name::new("Light Probe Sphere 1"),
  ));

  commands.spawn((
    Mesh3d(sphere_mesh.clone()),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::WHITE,
      metallic: 0.0,
      perceptual_roughness: 1.0,
      ..default()
    })),
    Transform::from_xyz(-1.0, 1.0, 1.0),
    Name::new("Light Probe Sphere 2"),
  ));

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
  _scattering_mediums: &mut Assets<ScatteringMedium>,
  _asset_server: &AssetServer,
) -> EntityCommands<'a> {
  // Sun
  commands.spawn((
    Sun,
    DirectionalLight {
      shadows_enabled: true,
      illuminance: if cfg!(feature = "atmosphere") {
        // lux::RAW_SUNLIGHT is recommended for use with this feature, since
        // other values approximate sunlight *post-scattering* in various
        // conditions. RAW_SUNLIGHT in comparison is the illuminance of the
        // sun unfiltered by the atmosphere, so it is the proper input for
        // sunlight to be filtered by the atmosphere.
        lux::RAW_SUNLIGHT
      } else {
        lux::AMBIENT_DAYLIGHT
      },
      ..default()
    },
    Transform::from_xyz(1.0, 0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    VolumetricLight,
    Name::new("Sun"),
  ));

  #[cfg(not(feature = "atmosphere"))]
  {
    // ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate color and brightness to match
    commands.insert_resource(GlobalAmbientLight {
      color: Color::srgb_u8(210, 220, 240),
      brightness: 1.0,
      ..default()
    });
  }

  // camera
  let mut cam = commands.spawn((
    Camera3d::default(),
    Hdr,
    Transform::from_xyz(-20.0, WATER_HEIGHT + 5.0, 20.0)
      .looking_at(Vec3::new(0.0, WATER_HEIGHT, 0.0), Vec3::Y),
    FreeCamera::default(),
  ));

  // spawn the fog volume as a child of the camera
  cam.with_children(|parent| {
    parent.spawn((
      FogVolume::default(),
      Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
      Name::new("Fog Volume"),
    ));
  });

  #[cfg(feature = "atmosphere")]
  {
    cam.insert((
      // Earthlike atmosphere
      Atmosphere::earthlike(_scattering_mediums.add(ScatteringMedium::default())),
      // Can be adjusted to change the scene scale and rendering quality
      AtmosphereSettings::default(),
      // The directional light illuminance used in this scene
      // (the one recommended for use with this feature) is
      // quite bright, so raising the exposure compensation helps
      // bring the scene to a nicer brightness range.
      Exposure { ev100: 13.0 },
      // Tonemapper chosen just because it looked good with the scene, any
      // tonemapper would be fine :)
      Tonemapping::AcesFitted,
      // Bloom gives the sun a much more natural look.
      Bloom::NATURAL,
      // Enables the atmosphere to drive reflections and ambient lighting (IBL) for this view
      AtmosphereEnvironmentMapLight::default(),
      FreeCamera::default(),
      VolumetricFog {
        ambient_intensity: 0.0,
        ..default()
      },
      Msaa::Off,
      Fxaa::default(),
    ));
  }
  #[cfg(not(feature = "atmosphere"))]
  {
    cam.insert((
      Msaa::Off,
      #[cfg(all(
        not(feature = "atmosphere"),
        not(target_arch = "wasm32"),
        not(feature = "ssr")
      ))]
      TemporalAntiAliasing::default(),
      ScreenSpaceAmbientOcclusion::default(),
      EnvironmentMapLight {
        diffuse_map: _asset_server
          .load("environment_maps/table_mountain_2_puresky_4k_diffuse.ktx2"),
        specular_map: _asset_server
          .load("environment_maps/table_mountain_2_puresky_4k_specular.ktx2"),
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
      Skybox {
        image: _asset_server.load("environment_maps/table_mountain_2_puresky_4k_cubemap.ktx2"),
        brightness: 2000.0,
        ..default()
      },
    ));
  }

  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }

  #[cfg(feature = "ssr")]
  {
    cam.insert((ScreenSpaceReflections::default(), Msaa::Off));
  }

  // This is just to keep the compiler happy when not using `depth_prepass` feature.
  cam.insert(Name::new("Camera"));

  cam
}

/// set up a simple 3D camera
pub fn setup_camera(
  mut commands: Commands,
  mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
  asset_server: Res<AssetServer>,
) {
  make_camera(&mut commands, &mut scattering_mediums, &asset_server);
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
