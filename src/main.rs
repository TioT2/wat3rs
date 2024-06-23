use std::collections::HashMap;

use animation::render::PrimitiveDescriptor;
use util::{rand::Xorshift32, Mat4x4f, Vec2f, Vec3f};

pub mod animation;
pub mod util;

struct Startup {
}

impl animation::Unit for Startup {
    fn response(&mut self, state: &mut animation::State) {
        // if let Some(model) = ObjModel::new(state, include_str!("../models/rei.obj").lines()) {
        //     state.add_unit(Box::new(model));
        // }
        // if let Some(model) = ObjModel::new(state, include_str!("../models/cow.obj").lines()) {
        //     state.add_unit(Box::new(model));
        // }
        // if let Some(model) = ObjModel::new(state, include_str!("../models/e1m1.obj").lines()) {
        //     state.add_unit(Box::new(model));
        // }

        state.add_unit(Box::new(CameraController::default()));

        {
            let w = Box::new(WaterSimulation::new(state));
            state.add_unit(w);
        }

        state.render.get_camera_mut().set(Vec3f::new(10.0, 10.0, 20.0), Vec3f::new(0.0, 0.0, 0.0), Vec3f::new(0.0, 1.0, 0.0));

        state.request_self_remove();
    }
}

struct WaterSimulationParticle {
    primitive: animation::render::Primitive,
    position: Vec3f,
    velocity: Vec3f,
}

struct WaterSimulation {
    bound_box_min: Vec3f,
    bound_box_max: Vec3f,
    bound_box_primitives: [animation::render::Primitive; 9],

    particle_primitive: animation::render::Primitive,

    particles: Vec<WaterSimulationParticle>,
}

impl WaterSimulation {
    fn create_obj_primitive<'t>(state: &mut animation::State, lines: impl Iterator<Item = &'t str>) -> (animation::render::Geometry, animation::render::GeometryAccessor, animation::render::GeometryAccessor) {
        let (vtx, idx) = util::obj::parse_obj(lines).unwrap();
        let mut geometry_buffer: Vec<u8> = Vec::new();

        geometry_buffer.extend_from_slice(bytemuck::cast_slice(vtx.as_slice()));
        geometry_buffer.extend_from_slice(bytemuck::cast_slice(idx.as_slice()));

        let geometry = state.render.create_geometry(&animation::render::GeometryDescriptor {
            geometry_buffer: &geometry_buffer,
            primitives: &[]
        });

        (
            geometry,
            animation::render::GeometryAccessor {
                offset: 0,
                size: vtx.len() * std::mem::size_of::<animation::render::Vertex>(),
                count: vtx.len() as u32,
            },
            animation::render::GeometryAccessor {
                offset: vtx.len() * std::mem::size_of::<animation::render::Vertex>(),
                size: idx.len() * std::mem::size_of::<u32>(),
                count: idx.len() as u32,
            },
        )
    }

    fn set_bound_box(&mut self, v1: Vec3f, v2: Vec3f) {
        self.bound_box_min.x = v1.x.min(v2.x);
        self.bound_box_min.y = v1.y.min(v2.y);
        self.bound_box_min.z = v1.z.min(v2.z);

        self.bound_box_max.x = v1.x.max(v2.x);
        self.bound_box_max.y = v1.y.max(v2.y);
        self.bound_box_max.z = v1.z.max(v2.z);

        let unit = Mat4x4f::scale((0.5, 0.5, 0.5).into()) * Mat4x4f::translate((0.5, 0.5, 0.5).into());

        // floor
        self.bound_box_primitives[0].set_transform(unit
            * Mat4x4f::scale(Vec3f::new(self.bound_box_max.x - self.bound_box_min.x + 2.0, 1.0, self.bound_box_max.z - self.bound_box_min.z + 2.0))
            * Mat4x4f::translate(Vec3f::new(self.bound_box_min.x - 1.0, self.bound_box_min.y - 1.0, self.bound_box_min.z - 1.0))
        );

        // vertical sticks
        let s_unit = unit * Mat4x4f::scale(Vec3f::new(1.0, self.bound_box_max.y - self.bound_box_min.y + 1.0, 1.0));
        self.bound_box_primitives[1].set_transform(s_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_max.x, self.bound_box_min.y, self.bound_box_max.z)));
        self.bound_box_primitives[2].set_transform(s_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_max.x, self.bound_box_min.y, self.bound_box_min.z - 1.0)));
        self.bound_box_primitives[3].set_transform(s_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_min.x - 1.0, self.bound_box_min.y, self.bound_box_max.z)));
        self.bound_box_primitives[4].set_transform(s_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_min.x - 1.0, self.bound_box_min.y, self.bound_box_min.z - 1.0)));

        let x_unit = unit * Mat4x4f::scale(Vec3f::new(self.bound_box_max.x - self.bound_box_min.x, 1.0, 1.0));
        self.bound_box_primitives[5].set_transform(x_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_min.x, self.bound_box_max.y, self.bound_box_max.z)));
        self.bound_box_primitives[6].set_transform(x_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_min.x, self.bound_box_max.y, self.bound_box_min.z - 1.0)));

        let y_unit = unit * Mat4x4f::scale(Vec3f::new(1.0, 1.0, self.bound_box_max.z - self.bound_box_min.z));
        self.bound_box_primitives[7].set_transform(y_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_max.x, self.bound_box_max.y, self.bound_box_min.z)));
        self.bound_box_primitives[8].set_transform(y_unit * Mat4x4f::translate(Vec3f::new(self.bound_box_min.x - 1.0, self.bound_box_max.y, self.bound_box_min.z)));
    }

    pub fn new(state: &mut animation::State) -> Self {
        let icosphere = Self::create_obj_primitive(state, include_str!("../models/icosphere.obj").lines());
        let cube = Self::create_obj_primitive(state, include_str!("../models/cube.obj").lines());

        let bound_primitive = cube.0.create_primitive(&animation::render::PrimitiveDescriptor {
            color: Vec3f::new(1.0, 1.0, 1.0),
            vertex_accessor: cube.1,
            index_accessor: Some(cube.2),
        });

        let particle_primitive = icosphere.0.create_primitive(&animation::render::PrimitiveDescriptor {
            color: Vec3f::new(1.0, 1.0, 1.0),
            vertex_accessor: icosphere.1,
            index_accessor: Some(icosphere.2),
        });
        particle_primitive.set_transform(Mat4x4f::scale(Vec3f::new(0.0, 0.0, 0.0)));

        let mut v = Self {
            bound_box_primitives: [
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive.clone(),
                bound_primitive,
            ],
            bound_box_min: Vec3f::default(),
            bound_box_max: Vec3f::default(),

            particles: vec![WaterSimulationParticle {
                position: Vec3f::new(0.0, 1.0, 0.0),
                velocity: Vec3f::new(3.0, 10.0, 1.5),
                primitive: icosphere.0.create_primitive(&animation::render::PrimitiveDescriptor {
                    color: Vec3f::new(1.0, 1.0, 1.0),
                    index_accessor: Some(icosphere.2),
                    vertex_accessor: icosphere.1,
                }),
            }],
            particle_primitive,
        };

        v.set_bound_box(Vec3f::new(-10.0, 0.0, -10.0), Vec3f::new(10.0, 20.0, 10.0));

        v
    }

    fn step(&mut self, dt: f32) {
        let gravity = Vec3f::new(0.0, -10.0, 0.0);

        // Yay, O(n^2)
        for i in 0..self.particles.len() {
            let p = self.particles.get(i).unwrap();

            let mut vel = p.velocity;
            let mut pos = p.position;

            for j in (i + 1)..self.particles.len() {
                let q = self.particles.get_mut(j).unwrap();

                let pvec = pos - q.position;
                let length2 = pvec.length2();

                if length2 <= 4.0 {
                    vel -= pvec * ((pvec ^ vel) * 2.0 / length2);
                    q.velocity -= pvec * ((pvec ^ q.velocity) * 2.0 / length2);

                    pos += pvec * 0.5;
                    q.position -= pvec * 0.5;
                }
            }

            let p = self.particles.get_mut(i).unwrap();

            p.velocity = vel;
            p.position = pos;

            p.velocity += gravity * dt;
            p.position += p.velocity * dt;

            if p.position.x <= self.bound_box_min.x + 1.0 { p.velocity.x *= -1.0; p.position.x = self.bound_box_min.x + 1.0; } else
            if p.position.x >= self.bound_box_max.x - 1.0 { p.velocity.x *= -1.0; p.position.x = self.bound_box_max.x - 1.0; }
            if p.position.y <= self.bound_box_min.y + 1.0 { p.velocity.y *= -1.0; p.position.y = self.bound_box_min.y + 1.0; } else
            if p.position.y >= self.bound_box_max.y - 1.0 { p.velocity.y *= -1.0; p.position.y = self.bound_box_max.y - 1.0; }
            if p.position.z <= self.bound_box_min.z + 1.0 { p.velocity.z *= -1.0; p.position.z = self.bound_box_min.z + 1.0; } else
            if p.position.z >= self.bound_box_max.z - 1.0 { p.velocity.z *= -1.0; p.position.z = self.bound_box_max.z - 1.0; }

            p.primitive.set_transform(Mat4x4f::translate(p.position));
        }
    }
}

impl animation::Unit for WaterSimulation {
    fn response(&mut self, state: &mut animation::State) {
        if state.input.is_key_clicked(animation::input::KeyCode::Space) {
            self.particles.push(WaterSimulationParticle {
                position: Vec3f {
                    x: (self.bound_box_max.x + self.bound_box_min.x) / 2.0,
                    y: self.bound_box_max.y - 1.0,
                    z: (self.bound_box_max.z + self.bound_box_min.z) / 2.0,
                },
                velocity: Vec3f::new(0.0, 0.0, 0.0),
                primitive: self.particle_primitive.clone(),
            })
        }

        self.step(state.timer.get_delta_time() as f32);
    }
}

struct ObjModel {
    geometry: animation::render::Geometry,
}

impl ObjModel {
    pub fn new<'t>(state: &mut animation::State, lines: impl Iterator<Item = &'t str>) -> Option<Self> {
        let (vtx, idx) = util::obj::parse_obj(lines)?;

        let mut geometry_buffer: Vec<u8> = Vec::new();
        geometry_buffer.extend_from_slice(bytemuck::cast_slice(vtx.as_slice()));
        geometry_buffer.extend_from_slice(bytemuck::cast_slice(idx.as_slice()));

        let geometry = state.render.create_geometry(&animation::render::GeometryDescriptor {
            geometry_buffer: &geometry_buffer,
            primitives: &[]
        });

        geometry.create_primitive(&animation::render::PrimitiveDescriptor {
            color: Vec3f::new(1.0, 1.0, 1.0),
            vertex_accessor: animation::render::GeometryAccessor {
                offset: 0,
                size: vtx.len() * std::mem::size_of::<animation::render::Vertex>(),
                count: vtx.len() as u32,
            },
            index_accessor: Some(animation::render::GeometryAccessor {
                offset: vtx.len() * std::mem::size_of::<animation::render::Vertex>(),
                size: idx.len() * std::mem::size_of::<u32>(),
                count: idx.len() as u32,
            }),
        });

        Some(ObjModel { geometry })
    }
}

impl animation::Unit for ObjModel {
    fn response(&mut self, state: &mut animation::State) {

    }
}

struct CameraController {
    pub movement_speed: f32,
    pub rotation_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            movement_speed: 8.0,
            rotation_speed: 2.0,
        }
    }
}

impl animation::Unit for CameraController {
    fn response(&mut self, state: &mut animation::State) {
        let move_axis = Vec3f {
            x: (state.input.is_key_pressed(animation::input::KeyCode::KeyD) as i32 - state.input.is_key_pressed(animation::input::KeyCode::KeyA) as i32) as f32,
            y: (state.input.is_key_pressed(animation::input::KeyCode::KeyR) as i32 - state.input.is_key_pressed(animation::input::KeyCode::KeyF) as i32) as f32,
            z: (state.input.is_key_pressed(animation::input::KeyCode::KeyW) as i32 - state.input.is_key_pressed(animation::input::KeyCode::KeyS) as i32) as f32,
        };

        let rotate_axis = Vec2f {
            x: (state.input.is_key_pressed(animation::input::KeyCode::ArrowRight) as i32 - state.input.is_key_pressed(animation::input::KeyCode::ArrowLeft) as i32) as f32,
            y: (state.input.is_key_pressed(animation::input::KeyCode::ArrowDown) as i32 - state.input.is_key_pressed(animation::input::KeyCode::ArrowUp) as i32) as f32,
        };

        if move_axis.length2() <= 0.01 && rotate_axis.length2() <= 0.01 {
            return;
        }

        let camera = state.render.get_camera_mut();
        let location = camera.get_location();

        let new_location = location.location + (
            location.right * move_axis.x +
            location.up * move_axis.y +
            location.direction * move_axis.z
        ) * state.timer.get_delta_time() as f32 * self.movement_speed;

        camera.set(
            new_location,
            new_location + {
                let rotation_delta = rotate_axis * state.timer.get_delta_time() as f32 * self.rotation_speed;

                let new_rotation = Vec2f {
                    x: location.direction.z.signum() * (
                        location.direction.x / (
                            location.direction.x * location.direction.x +
                            location.direction.z * location.direction.z
                        ).sqrt()
                    ).acos() + rotation_delta.x,
                    y: (location.direction.y.acos() + rotation_delta.y).clamp(0.01, std::f32::consts::PI - 0.01),
                };

                Vec3f {
                    x: new_rotation.y.sin() * new_rotation.x.cos(),
                    y: new_rotation.y.cos(),
                    z: new_rotation.y.sin() * new_rotation.x.sin(),
                }.normalized()
            },
            Vec3f::new(0.0, 1.0, 0.0)
        );
    } // fn response
}

fn main() {
    let mut anim = animation::Animation::new();
    anim.set_startup_unit(Box::new(Startup {}));
    _ = anim.run();
}
