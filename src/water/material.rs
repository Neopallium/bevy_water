use bevy::{
  asset::load_internal_asset,
  pbr::{ExtendedMaterial, MaterialExtension},
  prelude::*,
  reflect::{std_traits::ReflectDefault, Reflect},
  render::{render_asset::*, render_resource::*, texture::GpuImage},
};

pub type StandardWaterMaterial = ExtendedMaterial<StandardMaterial, WaterMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[uniform(100, WaterMaterialUniform)]
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

pub const NOISE_FBM_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x47c86614dedb33fe);

pub const NOISE_RANDOM_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x339ea286e4c7be3e);

pub const NOISE_VNOISE_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x2cb48f03a340aedc);

pub const WATER_BINDINGS_HANDLE: Handle<Shader> = Handle::weak_from_u128(0xa9010bab18132e4b);

pub const WATER_FUNCTIONS_HANDLE: Handle<Shader> = Handle::weak_from_u128(0xb73bf2f50994c394);

pub const WATER_VERTEX_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0xcea5177230c961ac);

pub const WATER_FRAGMENT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0xbe72b1f6760558cb);

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
  fn vertex_shader() -> ShaderRef {
    water_vertex_shader()
  }

  fn fragment_shader() -> ShaderRef {
    water_fragment_shader()
  }

  fn deferred_vertex_shader() -> ShaderRef {
    water_vertex_shader()
  }

  fn deferred_fragment_shader() -> ShaderRef {
    water_fragment_shader()
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
        ..default()
      })
      .register_asset_reflect::<StandardWaterMaterial>();
  }
}
