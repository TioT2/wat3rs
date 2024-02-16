/**/
struct Vertex {
    @location(0) position: vec3f,
    @location(1) texcoord: vec2f,
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
    @location(0) normal: vec3f,
}

@vertex
fn vs_main(vertex: Vertex) -> VsOut {
    var vs_out: VsOut;

    vs_out.projected_position = transform_arr[primitive_data.primitive_index].world_view_projection * vec4f(vertex.position.xyz, 1.0);
    vs_out.normal = transform_arr[primitive_data.primitive_index].world_inverse * vertex.normal;

    return vs_out;
}

@fragment
fn fs_main(vs_out: VsOut) -> @location(0) vec4f {
    return vec4f(primitive_data.base_color * abs(dot(normalize(vs_out.normal), normalize(vec3f(1.0, 1.0, 1.0)))), 255.0);
}