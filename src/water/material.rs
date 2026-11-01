use bevy::{
  mesh::MeshVertexBufferLayoutRef,
  pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
  prelude::*,
  reflect::{std_traits::ReflectDefault, Reflect},
  render::{render_asset::*, render_resource::*, texture::GpuImage},
  shader::*,
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
  /// Wave direction A (fading out during transition).
  pub wave_dir_a: Vec2,
  /// Wave direction B (fading in during transition).
  pub wave_dir_b: Vec2,
  /// Blend factor between directions: 0 = fully A, 1 = fully B.
  pub wave_blend: f32,
  pub quality: u32,
}

impl Default for WaterMaterial {
  fn default() -> Self {
    let default_dir = Vec2::new(1.0, 2.0).normalize();
    Self {
      clarity: 0.1,
      deep_color: Color::srgba(0.2, 0.41, 0.54, 1.0),
      shallow_color: Color::srgba(0.45, 0.78, 0.81, 1.0),
      edge_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
      edge_scale: 0.1,
      amplitude: 1.0,
      coord_offset: Vec2::new(0.0, 0.0),
      coord_scale: Vec2::new(1.0, 1.0),
      wave_dir_a: default_dir,
      wave_dir_b: default_dir,
      wave_blend: 1.0,
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
  pub wave_blend: f32,
  pub wave_dir_a: Vec2,
  pub wave_dir_b: Vec2,
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
      wave_dir_a: self.wave_dir_a,
      wave_dir_b: self.wave_dir_b,
      wave_blend: self.wave_blend,
    }
  }
}

const WATER_VERTEX_SHADER_PATH: &str = "embedded://bevy_water/shaders/water_vertex.wesl";
const WATER_FRAGMENT_SHADER_PATH: &str = "embedded://bevy_water/shaders/water_fragment.wesl";

fn water_fragment_shader() -> ShaderRef {
  WATER_FRAGMENT_SHADER_PATH.into()
}

fn water_vertex_shader() -> ShaderRef {
  WATER_VERTEX_SHADER_PATH.into()
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
    let quality = ShaderDefVal::UInt("QUALITY".into(), key.bind_group_data.quality);
    if let Some(fragment) = descriptor.fragment.as_mut() {
      // Bevy 0.20 sets `LOAD_PREPASS_NORMALS` for all materials when the view has
      // a `NormalPrepass` (e.g. required by SSAO), not just opaque ones.  The water
      // computes its own wave normals and isn't drawn in the normal prepass, so the
      // loaded normals would be from the geometry behind the water.
      fragment
        .shader_defs
        .retain(|def| def != &ShaderDefVal::from("LOAD_PREPASS_NORMALS"));
      fragment.shader_defs.push(quality.clone());
    }
    descriptor.vertex.shader_defs.push(quality);
    Ok(())
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
  fn build(&self, app: &mut App) {
    crate::shaders::load_shaders(app);

    app
      .add_plugins(MaterialPlugin::<StandardWaterMaterial>::default())
      .register_asset_reflect::<StandardWaterMaterial>();
  }
}
