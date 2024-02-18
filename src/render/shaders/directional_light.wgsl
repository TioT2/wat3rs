/// WAT3RS Project
/// `File` render/shaders/directional_light.wgsl
/// `Description` Directional lighting shader module.
/// `Author` TioT2
/// `Last changed` 18.02.2024

struct Camera {
    view_matrix: mat4x4f,
    projection_matrix: mat4x4f,
    view_projection_matrix: mat4x4f,

    location: vec3f,
    at: vec3f,

    direction: vec3f,
    right: vec3f,
    up: vec3f,
}

struct DirectionalLightData {
    direction: vec3f,
    power: f32,
    color: vec3f,
}

struct VsOut {
    @builtin(position) ndc_position: vec4f,
    @location(0) tex_coord: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
    var tex_coord = vec2f(
        f32(index / 2),
        f32(index % 2)
    );

    return VsOut(
        vec4f(tex_coord * 2.0 - 1.0, 0.0, 1.0),
        tex_coord * vec2f(1.0, -1.0)
    );
}

@group(0) @binding(0) var position_id: texture_2d<f32>;
@group(0) @binding(1) var normal_id: texture_2d<i32>;
@group(0) @binding(2) var color_opacity: texture_2d<f32>;
@group(0) @binding(3) var metallic_roughness_occlusion_meta: texture_2d<f32>;

@group(1) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(0) var<uniform> light_data: DirectionalLightData;

const I16_MAX_F = f32(32767);
const PI_F = 3.14159265357989;

fn ggx_partial_geometry(cos_theta_n: f32, alpha: f32) -> f32
{
    let cos_theta_sqr = cos_theta_n * cos_theta_n;
    let tan_sqr = (1.0 - cos_theta_sqr) / cos_theta_sqr;

    return 2.0 / (1.0 + sqrt(1 + alpha * alpha * tan_sqr));
} // fn ggx_partial_geometry

fn ggx_distribution(cos_theta_nh: f32, alpha: f32) -> f32
{
    let alpha_sqr = alpha * alpha;
    let nh_sqr = cos_theta_nh * cos_theta_nh;
    let den = nh_sqr * alpha_sqr + (1.0 - nh_sqr);
    return alpha_sqr / (PI_F * den * den);
} // fn ggx_distribution

fn frensel_schlick(f0: vec3f, cos_theta: f32) -> vec3f
{
  return f0 + (1.0 - f0) * pow(1.0 - saturate(cos_theta), 5.0);
} // fn frensel_schlick

@fragment
fn fs_main(@builtin(position) frag_coord_4f: vec4f, @location(0) tex_coord: vec2f) -> @location(0) vec4f {

    let frag_coord = vec2i(frag_coord_4f.xy);

    let position = textureLoad(position_id, frag_coord, 0).xyz;
    let normal = vec3f(textureLoad(normal_id, frag_coord, 0).xyz) / I16_MAX_F;
    let color_opacity = textureLoad(color_opacity, frag_coord, 0);
    let metallic_roughness_occlusion_meta = textureLoad(metallic_roughness_occlusion_meta, frag_coord, 0);

    // return vec4f(light_data.color * saturate(dot(normal, -light_data.direction)) * metallic_roughness_occlusion_meta.w, 1.0);

    let nl = dot(normal, -light_data.direction);

    let v = normalize(camera.location - position);
    let nv = dot(normal.xyz, v);

    let h = normalize(v - light_data.direction);
    let nh = dot(normal, h);
    let hv = dot(h, v);
    let roughness_2 = metallic_roughness_occlusion_meta.y * metallic_roughness_occlusion_meta.y;

    let g = ggx_partial_geometry(nv, roughness_2) * ggx_partial_geometry(nl, roughness_2);
    let d = ggx_distribution(nh, roughness_2);
    let f = frensel_schlick(color_opacity.xyz, hv);

    let spec = f * d * g * 0.25 / (nv + 0.01);
    let diff = color_opacity.xyz * saturate(1.0 - f) * nl / PI_F;

    return vec4f(max(spec + diff, vec3f(0.0)) * light_data.color * metallic_roughness_occlusion_meta.w * f32(nl > 0) * f32(nv > 0), 1.0);
} // fn fs_main

// file directional_light.wgsl
