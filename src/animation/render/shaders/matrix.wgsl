/// WAT3RS Project
/// `File` render/matrix.wgsl
/// `Description` Matrix compute shader script.
/// `Author` TioT2
/// `Last changed` 17.02.2024

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

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<storage, read> world_arr: array<mat4x4f>;
@group(0) @binding(2) var<storage, read_write> transform_arr: array<Transform>;

fn trunc(m: mat4x4f) -> mat3x3f
{
    return mat3x3f
    (
        m[0][0], m[0][1], m[0][2],
        m[1][0], m[1][1], m[1][2],
        m[2][0], m[2][1], m[2][2]
    );
}

fn inverse(m: mat3x3f) -> mat3x3f
{
    let det =
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) -
        m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) +
        m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);

    return mat3x3f
    (
        m[1][1] * m[2][2] - m[1][2] * m[2][1], m[1][0] * m[2][2] - m[1][2] * m[2][0], m[1][0] * m[2][1] - m[1][1] * m[2][0],
        m[0][1] * m[2][2] - m[0][2] * m[2][1], m[0][0] * m[2][2] - m[0][2] * m[2][0], m[0][0] * m[2][1] - m[0][1] * m[2][0],
        m[0][1] * m[1][2] - m[0][2] * m[1][1], m[0][0] * m[1][2] - m[0][2] * m[1][0], m[0][0] * m[1][1] - m[0][1] * m[1][0]
    ) * (1.0 / det);
} // fn inverse

@compute
@workgroup_size(1, 1, 1)
fn cs_main(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    transform_arr[id.x].world_view_projection = camera.view_projection_matrix * world_arr[id.x];
    transform_arr[id.x].world_inverse = inverse(transpose(trunc(world_arr[id.x])));
}

// file matrix.wgsl
