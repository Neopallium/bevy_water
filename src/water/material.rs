use bevy::{
  asset::{load_internal_asset, uuid_handle},
  pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
  prelude::*,
  shader::*,
  reflect::{std_traits::ReflectDefault, Reflect},
  render::{
    render_asset::*, render_resource::*, texture::GpuImage,
  },
  mesh::MeshVertexBufferLayoutRef,
};

pub type StandardWaterMaterial = ExtendedMaterial<StandardMaterial, WaterMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[uniform(100, WaterMaterialUniform)]
#[bind_group_data(WaterMaterialKey)]
#[reflect(Default, Debug)]
pub struct WaterMaterial {
  /// Water clarity, 0.0 = invisible.
  pub clarity: f32,
  /// Water color at deepest level.
  pub deep_color: Color,
  /// Water color at shallow areas.
  pub shallow_color: Color,
  /// Color of the edge effect.
  pub edge_color: Color,
  /// Scale of the water edge effect.
  pub edge_scale: f32,
  /// Wave amplitude.
  pub amplitude: f32,
  pub coord_offset: Vec2,
  pub coord_scale: Vec2,
  pub quality: u32,
}

impl Default for WaterMaterial {
  fn default() -> Self {
    Self {
      clarity: 0.1,
      deep_color: Color::srgba(0.2, 0.41, 0.54, 1.0),
      shallow_color: Color::srgba(0.45, 0.78, 0.81, 1.0),
      edge_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
      edge_scale: 0.1,
      amplitude: 1.0,
      coord_offset: Vec2::new(0.0, 0.0),
      coord_scale: Vec2::new(1.0, 1.0),
      quality: 4,
    }
  }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct WaterMaterialKey {
  quality: u32,
}

impl From<&WaterMaterial> for WaterMaterialKey {
  fn from(material: &WaterMaterial) -> WaterMaterialKey {
    WaterMaterialKey {
      quality: material.quality,
    }
  }
}

#[derive(Clone, Default, ShaderType)]
pub struct WaterMaterialUniform {
  pub deep_color: Vec4,
  pub shallow_color: Vec4,
  pub edge_color: Vec4,
  pub coord_offset: Vec2,
  pub coord_scale: Vec2,
  pub amplitude: f32,
  pub clarity: f32,
  pub edge_scale: f32,
}

impl AsBindGroupShaderType<WaterMaterialUniform> for WaterMaterial {
  fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> WaterMaterialUniform {
    WaterMaterialUniform {
      amplitude: self.amplitude,
      clarity: self.clarity,
      deep_color: self.deep_color.to_linear().to_vec4(),
      shallow_color: self.shallow_color.to_linear().to_vec4(),
      edge_scale: self.edge_scale,
      edge_color: self.edge_color.to_linear().to_vec4(),
      coord_offset: self.coord_offset,
      coord_scale: self.coord_scale,
    }
  }
}

pub const NOISE_FBM_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-78e6-9e95-669f43d631f4");

pub const NOISE_RANDOM_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-76a1-a5eb-5eec34b09b99");

pub const NOISE_VNOISE_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-7000-b5eb-93ec377e4060");

pub const WATER_BINDINGS_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-7024-a6dc-1d4843336463");

pub const WATER_FUNCTIONS_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-7a6e-b944-022ae6bb4fa9");

pub const WATER_VERTEX_SHADER_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-73a2-9223-08d5fedf602c");

pub const WATER_FRAGMENT_SHADER_HANDLE: Handle<Shader> = uuid_handle!("01968d7d-6cec-756e-9dc8-ccf5afaf5bb0");

#[cfg(feature = "embed_shaders")]
fn water_fragment_shader() -> ShaderRef {
  WATER_FRAGMENT_SHADER_HANDLE.into()
}

#[cfg(not(feature = "embed_shaders"))]
fn water_fragment_shader() -> ShaderRef {
  "shaders/water_fragment.wgsl".into()
}

#[cfg(feature = "embed_shaders")]
fn water_vertex_shader() -> ShaderRef {
  WATER_VERTEX_SHADER_HANDLE.into()
}

#[cfg(not(feature = "embed_shaders"))]
fn water_vertex_shader() -> ShaderRef {
  "shaders/water_vertex.wgsl".into()
}

impl MaterialExtension for WaterMaterial {
  #[cfg(not(feature = "ssr"))]
  fn vertex_shader() -> ShaderRef {
    water_vertex_shader()
  }

  #[cfg(not(feature = "ssr"))]
  fn fragment_shader() -> ShaderRef {
    water_fragment_shader()
  }

  #[cfg(feature = "ssr")]
  fn deferred_vertex_shader() -> ShaderRef {
    water_vertex_shader()
  }

  #[cfg(feature = "ssr")]
  fn deferred_fragment_shader() -> ShaderRef {
    water_fragment_shader()
  }

  fn specialize(
    _pipeline: &MaterialExtensionPipeline,
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayoutRef,
    key: MaterialExtensionKey<Self>,
  ) -> Result<(), SpecializedMeshPipelineError> {
    let dyn_water = key.bind_group_data.quality > 2;
    if let Some(fragment) = descriptor.fragment.as_mut() {
      fragment
        .shader_defs
        .push(format!("QUALITY_{}", key.bind_group_data.quality).into());
      if dyn_water {
        fragment.shader_defs.push("DYN_WATER".into());
      }
    }
    if dyn_water {
      descriptor.vertex.shader_defs.push("DYN_WATER".into());
    }
    Ok(())
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
  fn build(&self, app: &mut App) {
    load_internal_asset!(
      app,
      NOISE_FBM_HANDLE,
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/noise/fbm.wgsl"),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      NOISE_RANDOM_HANDLE,
      concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/noise/random.wgsl"
      ),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      NOISE_VNOISE_HANDLE,
      concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/noise/vnoise.wgsl"
      ),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_BINDINGS_HANDLE,
      concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/water_bindings.wgsl"
      ),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_FUNCTIONS_HANDLE,
      concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/water_functions.wgsl"
      ),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_VERTEX_SHADER_HANDLE,
      concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/water_vertex.wgsl"
      ),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_FRAGMENT_SHADER_HANDLE,
      concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/water_fragment.wgsl"
      ),
      Shader::from_wgsl
    );

    app
      .add_plugins(MaterialPlugin::<StandardWaterMaterial> {
        #[cfg(all(feature = "depth_prepass", not(feature = "ssr")))]
        prepass_enabled: false,
        ..default()
      })
      .register_asset_reflect::<StandardWaterMaterial>();
  }
}
