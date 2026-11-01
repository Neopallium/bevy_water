import bevy_pbr::render::{
  mesh_functions,
  skinning,
  view_transformations::position_world_to_clip,
};
import package::shaders::water_functions as water_fn;

@if(PREPASS_PIPELINE)
import bevy_pbr::prepass::io::{Vertex, VertexOutput, decompress_vertex};
@else
import bevy_pbr::render::forward_io::{Vertex, VertexOutput, decompress_vertex};

@vertex
fn vertex(vertex_in: Vertex) -> VertexOutput {
  var out: VertexOutput;
  let vertex = decompress_vertex(vertex_in, vertex_in.instance_index);

  let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
@if(SKINNED)
  var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights, vertex_in.instance_index);
@else
  var model = mesh_world_from_local;

  // The prepass only has normals when writing a normal/deferred prepass.
  var world_normal = vec3<f32>(0.0, 1.0, 0.0);
@if(VERTEX_NORMALS && (!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS) && SKINNED)
  world_normal = skinning::skin_normals(model, vertex.normal);
@if(VERTEX_NORMALS && (!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS) && !SKINNED)
  world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex_in.instance_index);
@if(!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS)
  out.world_normal = world_normal;

  let world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

  // Add the wave height to the world position.
  var height = -0.5;
  if constants::QUALITY > 2 {
@if(VERTEX_UVS_A)
    let w_pos = water_fn::uv_to_coord(vertex.uv);
    // Meshes without UVs: wave coords are world-space XZ (same as the water tiles).
@else
    let w_pos = world_position.xz;
    height = water_fn::get_wave_height(w_pos);
  }

  out.world_position = world_position + vec4<f32>((world_normal * height), 0.);
  out.position = position_world_to_clip(out.world_position.xyz);

@if(VERTEX_UVS_A)
  out.uv = vertex.uv;
@if(VERTEX_UVS_B)
  out.uv_b = vertex.uv_b;

@if(VERTEX_TANGENTS && (!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS))
  out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
    model,
    vertex.tangent,
    vertex_in.instance_index
  );

@if(VERTEX_COLORS)
  out.color = vertex.color;

@if(VERTEX_OUTPUT_INSTANCE_INDEX)
  out.instance_index = vertex_in.instance_index;

@if(VISIBILITY_RANGE_DITHER)
  out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
    vertex_in.instance_index, mesh_world_from_local[3]);

  return out;
}
