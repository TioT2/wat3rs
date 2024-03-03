/// WAT3RS Project
/// `File` lib.rs
/// `Description` Main project file
/// `Author` TioT2
/// `Last changed` 17.02.2024

pub mod math;
pub mod render;
pub mod timer;
pub mod input;
pub mod anim;

use math::*;
use anim::*;
use wasm_bindgen::prelude::*;
use std::sync::Arc;

/// Parsed OBJ Data representation structure
struct ParsedObj {
    pub vertices: Vec<render::Vertex>,
    pub indices: Vec<u32>,
}

/// Obj file parsing function
/// * `source` - OBJ Source string
/// * Returns result, that contains parsed objects or error message
fn parse_obj(source: &str) -> Result<ParsedObj, String> {
    let mut positions = Vec::<Vec3f>::new();
    let mut tex_coords = Vec::<Vec2f>::new();
    let mut normals = Vec::<Vec3f>::new();

    positions.push(Vec3f::new(0.0, 0.0, 0.0));
    tex_coords.push(Vec2f::new(0.0, 0.0));
    normals.push(Vec3f::new(0.0, 0.0, 0.0));

    let mut vertex_index_map = std::collections::BTreeMap::<(u32, u32, u32), u32>::new();
    let mut vertices = Vec::<render::Vertex>::new();
    let mut indices = Vec::<u32>::new();

    for line in source.lines() {
        let mut blocks = line.split(' ');

        let block_type = match blocks.next() {
            Some(t) => t.trim(),
            None => continue,
        };

        match block_type {
            "v" => positions.push(match blocks.next().zip(blocks.next()).zip(blocks.next()) {
                Some(((sx, sy), sz)) => Vec3f::new(
                    sx.trim().parse::<f32>().unwrap_or(0.0),
                    sy.trim().parse::<f32>().unwrap_or(0.0),
                    sz.trim().parse::<f32>().unwrap_or(0.0)
                ),
                None => Vec3f::new(0.0, 0.0, 0.0)
            }),

            "vt" => tex_coords.push(match blocks.next().zip(blocks.next()) {
                Some((sx, sy)) => Vec2f::new(
                    sx.trim().parse::<f32>().unwrap_or(0.0),
                    sy.trim().parse::<f32>().unwrap_or(0.0),
                ),
                None => Vec2f::new(0.0, 0.0)
            }),

            "vn" => normals.push(match blocks.next().zip(blocks.next()).zip(blocks.next()) {
                Some(((sx, sy), sz)) => Vec3f::new(
                    sx.trim().parse::<f32>().unwrap_or(0.0),
                    sy.trim().parse::<f32>().unwrap_or(0.0),
                    sz.trim().parse::<f32>().unwrap_or(0.0)
                ),
                None => Vec3f::new(0.0, 0.0, 0.0)
            }),

            "f" => {
                // Index from blocks iterator
                if blocks.clone().count() == 4 {

                }
                let mut idx = blocks.map(|vertex| {
                    let mut vti = vertex.split('/');

                    let tup = (
                        vti.next().map_or(0, |str| str.trim().parse::<u32>().unwrap_or(0)),
                        vti.next().map_or(0, |str| str.trim().parse::<u32>().unwrap_or(0)),
                        vti.next().map_or(0, |str| str.trim().parse::<u32>().unwrap_or(0))
                    );

                    if let Some(entry) = vertex_index_map.get(&tup) {
                        *entry
                    } else {
                        let i = vertices.len() as u32;
                        vertex_index_map.insert(tup, i);
                        vertices.push(render::Vertex {
                            position: *positions.get(tup.0 as usize).unwrap_or(&Vec3f::new(0.0, 0.0, 0.0)),
                            tex_coord: *tex_coords.get(tup.1 as usize).unwrap_or(&Vec2f::new(0.0, 0.0)),
                            normal: *normals.get(tup.2 as usize).unwrap_or(&Vec3f::new(0.0, 0.0, 0.0)),
                        });
                        i
                    }
                });

                let ibase = idx.next().unwrap();
                let mut i = idx.next().unwrap();

                'face_parsing: loop {
                    let inew = match idx.next() {
                        Some(i) => i,
                        None => break 'face_parsing
                    };

                    indices.push(ibase);
                    indices.push(i);
                    indices.push(inew);

                    i = inew;
                }
            },

            _ => {},
        }
    }

    Ok(ParsedObj {
        vertices,
        indices
    })
}

struct CameraController {

}

impl Unit for CameraController {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        let camera = state.render.lock_camera();
        let input_state = state.input_state;
        let delta_time = state.timer.get_delta_time();

        let move_axis = Vec3f::new(
            (input_state.is_key_pressed(input::KeyCode::KeyD) as i32 - input_state.is_key_pressed(input::KeyCode::KeyA) as i32) as f32,
            (input_state.is_key_pressed(input::KeyCode::KeyR) as i32 - input_state.is_key_pressed(input::KeyCode::KeyF) as i32) as f32,
            (input_state.is_key_pressed(input::KeyCode::KeyW) as i32 - input_state.is_key_pressed(input::KeyCode::KeyS) as i32) as f32,
        );
        let rotate_axis = Vec2f::new(
          (input_state.is_key_pressed(input::KeyCode::ArrowRight) as i32 - input_state.is_key_pressed(input::KeyCode::ArrowLeft) as i32) as f32,
          (input_state.is_key_pressed(input::KeyCode::ArrowDown) as i32 - input_state.is_key_pressed(input::KeyCode::ArrowUp) as i32) as f32,
        );

        if move_axis.length() <= 0.01 && rotate_axis.length() <= 0.01 {
            return;
        }

        let camera_location= camera.get_location();
        let movement_delta = (
            camera_location.right     * move_axis.x +
            camera_location.up        * move_axis.y +
            camera_location.direction * move_axis.z
        ) * delta_time * 8.0;

        let mut azimuth = camera_location.direction.y.acos();
        let mut elevator = camera_location.direction.z.signum() * (
            camera_location.direction.x / (
                camera_location.direction.x * camera_location.direction.x +
                camera_location.direction.z * camera_location.direction.z
            ).sqrt()
        ).acos();

        elevator += rotate_axis.x * delta_time * 2.0;
        azimuth += rotate_axis.y * delta_time * 2.0;

        azimuth = azimuth.clamp(0.01, std::f32::consts::PI - 0.01);

        let new_direction = Vec3f{
            x: azimuth.sin() * elevator.cos(),
            y: azimuth.cos(),
            z: azimuth.sin() * elevator.sin()
        };

        camera.set(&(camera_location.location + movement_delta), &(camera_location.location + movement_delta + new_direction), &Vec3f {x: 0.0, y: 1.0, z: 0.0});
    }
}

struct LightData {
    light: render::DirectionalLight,
    enabled: bool,
}

struct Light {
    data: Vec<LightData>,
}

impl Light {
    pub fn new(render: &mut render::Render) -> Light {
        let sources = [
            (render::DirectionalLightDescriptor {
                color: Vec3f::new(1.0, 1.0, 1.0),
                direction: -Vec3f::new(0.30, 0.47, 0.80),
                power: 1.0,
            }, false),
            (render::DirectionalLightDescriptor {
                color: Vec3f::new(0.0, 1.0, 1.0),
                direction: -Vec3f::new(0.30, 0.47, 0.80),
                power: 1.0,
            }, true),
            (render::DirectionalLightDescriptor {
                color: Vec3f::new(1.0, 0.0, 0.0),
                direction: -Vec3f::new(0.30, 0.47, -0.80),
                power: 1.0,
            }, true),
            (render::DirectionalLightDescriptor {
                color: Vec3f::new(1.0, 1.0, 0.0),
                direction: -Vec3f::new(-0.30, 0.47, -0.80),
                power: 1.0,
            }, true),
        ];
        Light {
            data: sources.iter().map(|(descriptor, enabled)| {
                LightData {
                    enabled: *enabled,
                    light: render.create_directional_light(descriptor),
                }
            }).collect()
        }
    }
}

impl Unit for Light {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        let digits = [input::KeyCode::Digit0, input::KeyCode::Digit1, input::KeyCode::Digit2, input::KeyCode::Digit3];

        for (light, digit) in self.data.iter_mut().zip(digits.iter()) {
            if state.input_state.is_key_clicked(*digit) {
                light.enabled = !light.enabled;
            }
            if light.enabled {
                state.frame.add_directional_light(&mut light.light);
            }
        }
    }
}

struct Model {
    primitive: render::Primitive,
}

impl Model {
    fn new(render: &mut render::Render, obj_pipeline: Arc<render::Pipeline>, data: &ParsedObj) -> Self {
        let mut primitive = render.create_primitive(&render::PrimitiveDescriptor {
            pipeline: obj_pipeline,
            indices: Some(&data.indices),
            vertices: &[
                unsafe {
                    std::slice::from_raw_parts(std::mem::transmute(data.vertices.as_ptr()), data.vertices.len() * std::mem::size_of::<render::Vertex>())
                }
            ],
            material: &render::Material {
                base_color: Vec3f::new(1.00, 1.00, 1.00),
                metallic: 1.0,
                roughness: 0.5,
            }
        });

        primitive.lock_transforms().push(Mat4x4f::identity());

        Self { primitive }
    }
}

impl Unit for Model {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        state.frame.add_primitive(&mut self.primitive);
    }
}

struct Root {
    added: bool,
    obj_pipeline: Option<Arc<render::Pipeline>>,
}

impl Root {
    pub fn new() -> Root {
        Root { added: false, obj_pipeline: None, }
    }
}

impl Unit for Root {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        if !self.added {
            state.scene_context.add_unit(Box::new(CameraController{}));

            self.obj_pipeline = state.render.create_primitive_pipeline(&render::PipelineDescriptor {
                shader_source: include_str!("render/shaders/primitive.wgsl"),
                vertices: &[render::VertexBufferDescriptor {
                    attributes: &[
                        render::VertexAttributeDescriptor { format: render::VertexFormat::Float32x3, location: 0, offset:  0, },
                        render::VertexAttributeDescriptor { format: render::VertexFormat::Float32x2, location: 1, offset: 12, },
                        render::VertexAttributeDescriptor { format: render::VertexFormat::Float32x3, location: 2, offset: 20, },
                    ],
                    stride: 32,
                }],
                polygon_mode: render::PolygonMode::Fill,
            }).ok();

            if let Some(obj_pipeline) = &self.obj_pipeline {
                state.scene_context.add_unit(Box::new(Light::new(state.render)));
                state.scene_context.add_unit(Box::new(Model::new(state.render, obj_pipeline.clone(),  &parse_obj(include_str!("../models/rei.obj")).unwrap())));
                state.scene_context.add_unit(Box::new(Model::new(state.render, obj_pipeline.clone(),  &parse_obj(include_str!("../models/e1m1.obj")).unwrap())));
                state.scene_context.add_unit(Box::new(Model::new(state.render, obj_pipeline.clone(),  &parse_obj(include_str!("../models/cow.obj")).unwrap())));
            }

            self.added = true;
        }
    }
}

#[wasm_bindgen]
pub fn run() {
    let mut system = System::new().unwrap();

    system.get_scene().add_unit(Box::new(Root::new()));

    system.run();
} // fn main

// file main.rs
