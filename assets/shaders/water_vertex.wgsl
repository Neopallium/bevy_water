#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::mesh_functions

#import bevy_water::water_bindings
#import bevy_water::water_functions

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

struct VertexOutput {
  @builtin(position) frag_coord: vec4<f32>,
  #import bevy_pbr::mesh_vertex_output
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;

#ifdef SKINNED
  var model = skin_model(vertex.joint_indices, vertex.joint_weights);
  out.world_normal = skin_normals(model, vertex.normal);
#else
  var model = mesh.model;
  out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif

  let world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

  // Add the wave height to the world position.
  let w_pos = uv_to_coord(vertex.uv);
  let height = get_wave_height(w_pos);

  out.world_position = world_position + vec4<f32>((out.world_normal * height), 0.);
  out.frag_coord = mesh_position_world_to_clip(out.world_position);

#ifdef VERTEX_UVS
  out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
  out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif

#ifdef VERTEX_COLORS
  out.color = vertex.color;
#endif

  return out;
}
