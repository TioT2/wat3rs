/// WAT3RS Project
/// `File` render/primitive.wgsl
/// `Description` Primitive shader script.
/// `Author` TioT2
/// `Last changed` 17.02.2024

struct Vertex {
    @location(0) position: vec3f,
    @location(1) tex_coord: vec2f,
    @location(2) normal: vec3f,
}

struct Transform {
    world_view_projection: mat4x4f,
    world_inverse: mat3x3f,
}

struct Camera {
    view_matrix: mat4x4f,
    projection_matrix: mat4x4f,
    view_projection_matrix: mat4x4f,

    camera_location: vec3f,
    camer_at: vec3f,

    camera_direction: vec3f,
    camera_right: vec3f,
    camera_up: vec3f,
}

struct PrimitiveData {
    base_color: vec3f,
    primitive_index: u32,
    roughness: f32,
    metallic: f32,
}


@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<storage, read> world_arr: array<mat4x4f>;
@group(0) @binding(2) var<storage, read> transform_arr: array<Transform>;
@group(1) @binding(0) var<uniform> primitive_data: PrimitiveData;

struct VsOut {
    @builtin(position) projected_position: vec4<f32>,
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) instance_id: u32,
}

@vertex
fn vs_main(vertex: Vertex, @builtin(instance_index) instance_id: u32) -> VsOut {
    var vs_out: VsOut;

    vs_out.projected_position = transform_arr[primitive_data.primitive_index].world_view_projection * vec4f(vertex.position.xyz, 1.0);
    vs_out.position = (world_arr[primitive_data.primitive_index] * vec4f(vertex.position.xyz, 1.0)).xyz;
    vs_out.normal = transform_arr[primitive_data.primitive_index].world_inverse * vertex.normal;
    vs_out.instance_id = instance_id;

    return vs_out;
}

struct FsOut {
    @location(0) position_id: vec4f,
    @location(1) normal_id: vec4i,
    @location(2) color_opacity: vec4f,
    @location(3) metallic_roughness_occlusion_meta: vec4f,
}

const I16_MAX_F = f32(32767);

@fragment
fn fs_main(vs_out: VsOut) -> FsOut {
    var out: FsOut;

    out.position_id = vec4f(vs_out.position, 1.0);
    out.normal_id = vec4i(vec3i((normalize(vs_out.normal.xyz)) * I16_MAX_F), 1);
    out.color_opacity = vec4f(primitive_data.base_color, 1.0);
    out.metallic_roughness_occlusion_meta = vec4f(primitive_data.metallic, primitive_data.roughness, 1.0, 1.0);

    return out;
} // fn fs_main

// file primitive.wgsl
