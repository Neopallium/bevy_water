#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::skinning
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput

#import bevy_water::water_functions as water_fn

struct Vertex {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
  @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
  @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
  @location(5) joint_indices: vec4<u32>,
  @location(6) joint_weights: vec4<f32>,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
  var out: MeshVertexOutput;

#ifdef SKINNED
  var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
  out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
#else
  var model = mesh.model;
  out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
#endif

  let world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

  // Add the wave height to the world position.
  let w_pos = water_fn::uv_to_coord(vertex.uv);
  let height = water_fn::get_wave_height(w_pos);

  out.world_position = world_position + vec4<f32>((out.world_normal * height), 0.);
  out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);

#ifdef VERTEX_UVS
  out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
  out.world_tangent = mesh_functions::mesh_tangent_local_to_world(model, vertex.tangent);
#endif

#ifdef VERTEX_COLORS
  out.color = vertex.color;
#endif

  return out;
}
