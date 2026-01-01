//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

#[cfg(feature = "debug")]
use bevy::color::palettes::css::*;
use bevy::prelude::*;

mod pirates;
use pirates::*;

fn main() {
  let mut app = pirates_app("Pirates with water debug lines");

  #[cfg(feature = "debug")]
  app.add_systems(Update, debug_water_wave_grid);

  // Setup
  app.add_systems(Startup, (setup_ocean, setup_orb, setup_camera, setup_ships));

  app.run();
}

#[cfg(feature = "debug")]
fn debug_water_wave_grid(
  water: WaterParam,
  tiles: Query<(
    &Handle<Mesh>,
    &Handle<StandardWaterMaterial>,
    &GlobalTransform,
  )>,
  meshes: Res<Assets<Mesh>>,
  materials: Res<Assets<StandardWaterMaterial>>,
  mut gizmos: Gizmos,
) {
  for (mesh, tile, global) in tiles.iter() {
    let mesh = meshes.get(mesh);
    let tile = materials.get(tile);
    match (mesh, tile) {
      (Some(mesh), Some(tile)) => {
        /*
        let coord_offset = tile.extension.coord_offset - Vec2::new(128.0, 128.0);
        let coord_scale = tile.extension.coord_scale;
        match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
          Some(VertexAttributeValues::Float32x2(uvs)) => {
            let length = 0.5;
            for [x, y] in uvs.iter() {
              let uv = Vec2::new(*x, *y);
              let coord = coord_offset + (uv * coord_scale);
              if coord.x > -110.0 || coord.y > -110.0 {
                continue;
              }
              let coord_pos = Vec3::new(coord.x, 0.0, coord.y);
              let coord_pos2 = Vec3::new(coord.x, 0.0, coord.y + length);
              let coord_pos3 = Vec3::new(coord.x + length, 0.0, coord.y);
              let w_pos = water.wave_point(global.transform_point(coord_pos));
              let w_pos2 = water.wave_point(global.transform_point(coord_pos2));
              let w_pos3 = water.wave_point(global.transform_point(coord_pos3));
              lines.line_colored(w_pos, w_pos2, 0.0, Color::RED);
              lines.line_colored(w_pos, w_pos3, 0.0, Color::GREEN);
            }
          }
          _ => {
            panic!("Unsupported or missing UVs");
          }
        }
        // */
        //*
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
          Some(VertexAttributeValues::Float32x3(pos)) => {
            let length = 1.0;
            for [x, _y, z] in pos.iter() {
              let x = *x;
              let z = *z;
              if x > -110.0 || z > -110.0 {
                continue;
              }
              let coord = Vec2::new(x, z);
              let coord_pos = Vec3::new(coord.x, 0.0, coord.y);
              let coord_pos2 = Vec3::new(coord.x, 0.0, coord.y + length);
              let coord_pos3 = Vec3::new(coord.x + length, 0.0, coord.y);
              let w_pos = water.wave_point(global.transform_point(coord_pos));
              let w_pos2 = water.wave_point(global.transform_point(coord_pos2));
              let w_pos3 = water.wave_point(global.transform_point(coord_pos3));
              lines.line_colored(w_pos, w_pos2, 0.0, Color::RED);
              lines.line_colored(w_pos, w_pos3, 0.0, Color::GREEN);
            }
          }
          _ => {
            panic!("Unsupported or missing vertices");
          }
        }
        // */
      }
      _ => (),
    }
  }
}
