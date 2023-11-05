use bevy::{
  asset::load_internal_asset,
  pbr::{MaterialPipeline, MaterialPipelineKey, StandardMaterialFlags, PBR_PREPASS_SHADER_HANDLE},
  prelude::*,
  reflect::{std_traits::ReflectDefault, Reflect, TypeUuid},
  render::{mesh::MeshVertexBufferLayout, render_asset::*, render_resource::*},
};

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "ea9ad5fe-a9ce-4ff1-aea8-d756ed456c46"]
#[bind_group_data(WaterMaterialKey)]
#[uniform(0, WaterMaterialUniform)]
#[reflect(Default, Debug)]
pub struct WaterMaterial {
  // StandardMaterial fields.
  pub base_color: Color,
  #[texture(1)]
  #[sampler(2)]
  pub base_color_texture: Option<Handle<Image>>,
  pub emissive: Color,
  #[texture(3)]
  #[sampler(4)]
  pub emissive_texture: Option<Handle<Image>>,
  pub perceptual_roughness: f32,
  pub metallic: f32,
  #[texture(5)]
  #[sampler(6)]
  pub metallic_roughness_texture: Option<Handle<Image>>,
  pub reflectance: f32,
  #[texture(9)]
  #[sampler(10)]
  pub normal_map_texture: Option<Handle<Image>>,
  pub flip_normal_map_y: bool,
  #[texture(7)]
  #[sampler(8)]
  pub occlusion_texture: Option<Handle<Image>>,
  pub double_sided: bool,
  #[reflect(ignore)]
  pub cull_mode: Option<Face>,
  pub unlit: bool,
  pub fog_enabled: bool,
  pub alpha_mode: AlphaMode,
  pub depth_bias: f32,
  // Water fields.
  /// Wave amplitude.
  pub amplitude: f32,
  /// Water clarity, 0.0 = invisible.
  pub clarity: f32,
  /// Water color at deepest level.
  pub deep_color: Color,
  /// Water color at shallow areas.
  pub shallow_color: Color,
  /// Scale of the water edge effect.
  pub edge_scale: f32,
  /// Color of the edge effect.
  pub edge_color: Color,
  pub coord_offset: Vec2,
  pub coord_scale: Vec2,
}

impl Default for WaterMaterial {
  fn default() -> Self {
    Self {
      // StandardMaterial fields.
      base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
      base_color_texture: None,
      emissive: Color::BLACK,
      emissive_texture: None,
      perceptual_roughness: 0.22,
      metallic: 0.0,
      metallic_roughness_texture: None,
      reflectance: 0.5,
      occlusion_texture: None,
      normal_map_texture: None,
      flip_normal_map_y: false,
      double_sided: false,
      cull_mode: Some(Face::Back),
      unlit: false,
      fog_enabled: true,
      alpha_mode: AlphaMode::Blend,
      depth_bias: 0.0,
      // WaterMaterial fields.
      amplitude: 1.0,
      clarity: 0.1,
      deep_color: Color::rgba(0.2, 0.41, 0.54, 1.0),
      shallow_color: Color::rgba(0.45, 0.78, 0.81, 1.0),
      edge_scale: 0.1,
      edge_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
      coord_offset: Vec2::new(0.0, 0.0),
      coord_scale: Vec2::new(1.0, 1.0),
    }
  }
}

#[derive(Clone, Default, ShaderType)]
pub struct WaterMaterialUniform {
  // StandardMaterial fields.
  pub base_color: Vec4,
  pub emissive: Vec4,
  pub roughness: f32,
  pub metallic: f32,
  pub reflectance: f32,
  pub flags: u32,
  pub alpha_cutoff: f32,
  // WaterMaterial fields.
  pub amplitude: f32,
  pub clarity: f32,
  pub edge_scale: f32,
  pub deep_color: Color,
  pub shallow_color: Color,
  pub edge_color: Color,
  pub coord_offset: Vec2,
  pub coord_scale: Vec2,
}

impl AsBindGroupShaderType<WaterMaterialUniform> for WaterMaterial {
  fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> WaterMaterialUniform {
    let mut flags = StandardMaterialFlags::NONE;
    if self.base_color_texture.is_some() {
      flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
    }
    if self.emissive_texture.is_some() {
      flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
    }
    if self.metallic_roughness_texture.is_some() {
      flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
    }
    if self.occlusion_texture.is_some() {
      flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
    }
    if self.double_sided {
      flags |= StandardMaterialFlags::DOUBLE_SIDED;
    }
    if self.unlit {
      flags |= StandardMaterialFlags::UNLIT;
    }
    if self.fog_enabled {
      flags |= StandardMaterialFlags::FOG_ENABLED;
    }
    let has_normal_map = self.normal_map_texture.is_some();
    if has_normal_map {
      if let Some(texture) = images.get(self.normal_map_texture.as_ref().unwrap()) {
        match texture.texture_format {
          // All 2-component unorm formats
          TextureFormat::Rg8Unorm
          | TextureFormat::Rg16Unorm
          | TextureFormat::Bc5RgUnorm
          | TextureFormat::EacRg11Unorm => {
            flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
          }
          _ => {}
        }
      }
      if self.flip_normal_map_y {
        flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
      }
    }
    // NOTE: 0.5 is from the glTF default - do we want this?
    let mut alpha_cutoff = 0.5;
    match self.alpha_mode {
      AlphaMode::Opaque => flags |= StandardMaterialFlags::ALPHA_MODE_OPAQUE,
      AlphaMode::Mask(c) => {
        alpha_cutoff = c;
        flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
      }
      AlphaMode::Blend => flags |= StandardMaterialFlags::ALPHA_MODE_BLEND,
      AlphaMode::Premultiplied => flags |= StandardMaterialFlags::ALPHA_MODE_PREMULTIPLIED,
      AlphaMode::Add => flags |= StandardMaterialFlags::ALPHA_MODE_ADD,
      AlphaMode::Multiply => flags |= StandardMaterialFlags::ALPHA_MODE_MULTIPLY,
    };

    WaterMaterialUniform {
      base_color: self.base_color.as_linear_rgba_f32().into(),
      emissive: self.emissive.as_linear_rgba_f32().into(),
      roughness: self.perceptual_roughness,
      metallic: self.metallic,
      reflectance: self.reflectance,
      flags: flags.bits(),
      alpha_cutoff,
      // WaterMaterial fields.
      amplitude: self.amplitude,
      clarity: self.clarity,
      deep_color: self.deep_color,
      shallow_color: self.shallow_color,
      edge_scale: self.edge_scale,
      edge_color: self.edge_color,
      coord_offset: self.coord_offset,
      coord_scale: self.coord_scale,
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WaterMaterialKey {
  normal_map: bool,
  cull_mode: Option<Face>,
  depth_bias: i32,
}

impl From<&WaterMaterial> for WaterMaterialKey {
  fn from(material: &WaterMaterial) -> Self {
    WaterMaterialKey {
      normal_map: material.normal_map_texture.is_some(),
      cull_mode: material.cull_mode,
      depth_bias: material.depth_bias as i32,
    }
  }
}

pub const NOISE_FBM_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0x47c86614dedb33fe);

pub const NOISE_RANDOM_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0x339ea286e4c7be3e);

pub const NOISE_VNOISE_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0x2cb48f03a340aedc);

pub const WATER_BINDINGS_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0xa9010bab18132e4b);

pub const WATER_FUNCTIONS_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0xb73bf2f50994c394);

pub const WATER_VERTEX_SHADER_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0xcea5177230c961ac);

pub const WATER_FRAGMENT_SHADER_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(0xbe72b1f6760558cb);

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

impl Material for WaterMaterial {
  fn specialize(
    _pipeline: &MaterialPipeline<Self>,
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayout,
    key: MaterialPipelineKey<Self>,
  ) -> Result<(), SpecializedMeshPipelineError> {
    if let Some(fragment) = descriptor.fragment.as_mut() {
      if key.bind_group_data.normal_map {
        fragment
          .shader_defs
          .push("STANDARDMATERIAL_NORMAL_MAP".into());
      }

      #[cfg(feature = "depth_prepass")]
      fragment
        .shader_defs
        .push("USE_DEPTH".into());
    }
    descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
    if let Some(label) = &mut descriptor.label {
      *label = format!("pbr_{}", *label).into();
    }
    if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
      depth_stencil.bias.constant = key.bind_group_data.depth_bias;
    }
    Ok(())
  }

  fn prepass_fragment_shader() -> ShaderRef {
    PBR_PREPASS_SHADER_HANDLE.into()
  }

  fn vertex_shader() -> ShaderRef {
    water_vertex_shader()
  }

  fn fragment_shader() -> ShaderRef {
    water_fragment_shader()
  }

  #[inline]
  fn alpha_mode(&self) -> AlphaMode {
    self.alpha_mode
  }

  #[inline]
  fn depth_bias(&self) -> f32 {
    self.depth_bias
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
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/water_bindings.wgsl"),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_FUNCTIONS_HANDLE,
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/water_functions.wgsl"),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_VERTEX_SHADER_HANDLE,
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/water_vertex.wgsl"),
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_FRAGMENT_SHADER_HANDLE,
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/water_fragment.wgsl"),
      Shader::from_wgsl
    );

    app.add_plugins(MaterialPlugin::<WaterMaterial> {
      prepass_enabled: false,
      ..default()
    })
      .register_type::<WaterMaterial>()
      .register_asset_reflect::<WaterMaterial>()
      .register_type::<Handle<WaterMaterial>>();
  }
}
