use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::*};

pub fn grid_mesh(size: u16, scale: f32, uv_scale: f32) -> Mesh {
  assert!(size <= 256, "Can't generate grid mesh with size > 256");
  let cap = (size as usize).pow(2);

  // Pre-calculate position & uv units.
  let uv_step = 1.0 / (size - 1) as f32;
  let pos_uv = (0..size)
    .map(|n| {
      let uv = uv_step * n as f32;
      let pos = uv * size as f32;
      (pos, uv)
    })
    .collect::<Vec<_>>();

  let mut positions = Vec::with_capacity(cap);
  let mut normals = Vec::with_capacity(cap);
  let mut uvs = Vec::with_capacity(cap);

  for (pos_x, uv_x) in pos_uv.iter() {
    for (pos_z, uv_z) in pos_uv.iter() {
      let pos = [*pos_x * scale, 0.0, *pos_z * scale];
      let uv = [(*uv_x) * uv_scale, (1.0 - *uv_z) * uv_scale];
      positions.push(pos);
      normals.push([0.0, 1.0, 0.0]);
      uvs.push(uv);
    }
  }

  let idx_cap = (cap - 2) * 2;
  let mut indices = Vec::with_capacity(idx_cap);
  for row_idx in 0..size - 1 {
    let top_offset = row_idx * size;
    let btm_offset = top_offset + size;
    if row_idx > 0 {
      // Degenerate triangles.
      indices.push(top_offset);
      indices.push(btm_offset - 1);
    }
    for idx in (0..size).rev() {
      // Top vertices.
      indices.push(top_offset + idx);
      // Bottom vertices.
      indices.push(btm_offset + idx);
    }
  }

  let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
  mesh.set_indices(Some(Indices::U16(indices)));
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

  mesh
}
