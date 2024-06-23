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

struct PrimitiveData {
    world_view_projection: mat4x4f,

    world: mat4x4f,

    world_inverse: mat3x3f,
    world_inverse_placeholder: vec4f,

    base_color: vec3f,
    primitive_index: u32,
}

@group(0) @binding(0) var<uniform> primitive_data: PrimitiveData;

struct VsOut {
    @builtin(position) projected_position: vec4<f32>,
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
}

@vertex
fn vs_main(vertex: Vertex, @builtin(instance_index) instance_id: u32) -> VsOut {
    var vs_out: VsOut;

    vs_out.projected_position = primitive_data.world_view_projection * vec4f(vertex.position, 1.0);
    vs_out.position = (primitive_data.world * vec4f(vertex.position, 1.0)).xyz;
    vs_out.normal = normalize(primitive_data.world_inverse * vertex.normal);

    return vs_out;
}

@fragment
fn fs_main(vs_out: VsOut) -> @location(0) vec4f {
    let d = dot(normalize(vs_out.normal), normalize(vec3f(0.30, 0.47, 0.80)));
    return vec4f(primitive_data.base_color * (clamp(d, 0.1, 1.0) * 0.9 + clamp(-d, 0.1, 1.0) * 0.1), 0.0);
} // fn fs_main

// file primitive.wgsl
