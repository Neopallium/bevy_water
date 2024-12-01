//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

use bevy::pbr::{DefaultOpaqueRendererMethod, ScreenSpaceReflections};
use bevy::prelude::*;
use bevy::render::mesh::*;

mod pirates;
use pirates::*;

fn main() {
  let mut app = pirates_app("Pirates with TAA");

  app.insert_resource(DefaultOpaqueRendererMethod::deferred());

  // Setup
  app.add_systems(
    Startup,
    (setup_ocean, setup_shiny_orb, setup_camera_ssr, setup_ships),
  );

  app.run();
}

/// Setup some shiny objects.
fn setup_shiny_orb(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let shiny = MeshMaterial3d(materials.add(StandardMaterial {
    base_color: Color::srgba(0.1, 0.2, 0.4, 1.0),
    perceptual_roughness: 0.0,
    metallic: 1.0,
    reflectance: 1.0,
    ..default()
  }));
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
    shiny.clone(),
    Transform::from_xyz(-5.0, 10.0, -5.0),
  ));
  let box_mesh = Mesh3d({
    let mut mesh = Cuboid::from_length(1.0).mesh().build();
    mesh.generate_tangents().expect("tangents");
    meshes.add(mesh)
  });
  commands.spawn((
    Name::new(format!("Box")),
    box_mesh.clone(),
    shiny.clone(),
    Transform::from_xyz(0.0, 2.0, 0.0).with_scale(Vec3::new(10.0, 0.1, 10.0)),
  ));
}

/// Setup the camera with SSR.
fn setup_camera_ssr(mut commands: Commands, asset_server: Res<AssetServer>) {
  // camera
  let mut cam = make_camera(&mut commands, &asset_server);

  cam.insert((ScreenSpaceReflections::default(), Msaa::Off));
}
