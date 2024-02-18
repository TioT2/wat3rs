/// WAT3RS Project
/// `File` main.rs
/// `Description` Main project file
/// `Author` TioT2
/// `Last changed` 17.02.2024

pub mod math;

pub mod render;
pub mod timer;
pub mod input;

use std::sync::Arc;

use math::*;

pub struct SceneContext {
    added_units: Vec<Box<dyn Unit>>,
}

impl SceneContext {
    pub fn add_unit(&mut self, unit: Box<dyn Unit>) {
        self.added_units.push(unit);
    }
}

pub struct SystemState<'a, 'scene, 'render> where 'render: 'scene, 'scene: 'a {
    pub scene: &'a mut render::Scene<'scene>,
    pub timer: &'a timer::Timer,
    pub input_state: &'a input::State,
    pub scene_context: &'a mut SceneContext,
    pub render: &'a mut render::Render<'render>,
}

pub trait Unit {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's;
}

pub struct Scene {
    units: Vec<Box<dyn Unit>>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            units: Vec::new(),
        }
    }

    pub fn create_context(&self) -> SceneContext {
        SceneContext {
            added_units: Vec::new(),
        }
    }

    pub fn flush_context(&mut self, mut context: SceneContext) {
        self.units.append(&mut context.added_units);
    }

    pub fn add_unit(&mut self, unit: Box<dyn Unit>) {
        self.units.push(unit);
    }

    fn response<'a, 's, 'r>(&'a mut self, mut state: SystemState<'s, 'a, 'r>) {
        for unit in &mut self.units {
            unit.response(&mut state);
        }
    }
}

pub struct System<'a> {
    event_loop: winit::event_loop::EventLoop<()>,
    main_window: Arc<winit::window::Window>,
    input: input::Input,
    render: render::Render<'a>,
    timer: timer::Timer,

    scene: Scene,
}

impl<'a> System<'a> {
    pub fn new() -> Result<Self, String> {
        let event_loop = winit::event_loop::EventLoop::new().map_err(|err| err.to_string())?;
        let main_window = Arc::new(winit::window::WindowBuilder::new()
            .with_title("wat3rs")
            // .with_resizable(false)
            .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(800, 600)))
            .build(&event_loop).map_err(|err| err.to_string())?);

        Ok(Self {
            event_loop,
            render: render::Render::new(main_window.clone())?,
            main_window,
            input: input::Input::new(),
            timer: timer::Timer::new(),
            scene: Scene::new(),
        })
    }

    pub fn get_scene<'s>(&'s mut self) -> &'s mut Scene {
        &mut self.scene
    }

    pub fn run(mut self) {
        let mut frame_index = 0;

        _ = self.event_loop.run(move |event, target| {
            match event {
                winit::event::Event::WindowEvent { window_id, event } => if self.main_window.id() == window_id {
                    match event {
                        winit::event::WindowEvent::CloseRequested => target.exit(),
                        winit::event::WindowEvent::KeyboardInput { event, .. } => if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                            self.input.on_key_state_change(code, event.state == winit::event::ElementState::Pressed);
                        }
                        winit::event::WindowEvent::Resized(size) => self.render.resize(Vec2::<usize>::new(size.width as usize, size.height as usize)),
                        winit::event::WindowEvent::RedrawRequested => {
                            self.timer.response();
                            if self.timer.get_fps() > 1.0 {
                                if frame_index % (self.timer.get_fps().ceil() as u32) == 1 {
                                    println!("FPS: {}", self.timer.get_fps());
                                }
                            } else {
                                println!("Less, than 1 FPS");
                            }
                            frame_index += 1;

                            if self.input.get_state().is_key_clicked(input::KeyCode::F11) {
                                if self.main_window.fullscreen().is_some() {
                                    self.main_window.set_fullscreen(None);
                                } else {
                                    self.main_window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(self.main_window.current_monitor())));
                                }
                            }

                            // camera_keyboard_control(self.render.lock_camera(), self.input.get_state(), self.timer.get_delta_time());

                            let mut scene = self.render.create_scene();

                            let mut scene_context = self.scene.create_context();

                            self.scene.response(SystemState {
                                input_state: &self.input.get_state(),
                                scene: &mut scene,
                                render: &mut self.render,
                                timer: &self.timer,
                                scene_context: &mut scene_context,
                            });

                            self.render.render_scene(&scene);
                            self.scene.flush_context(scene_context);

                            self.main_window.request_redraw();
                            self.input.clear_changed();
                        }
                        _ => {},
                    }
                }
                // Render
                winit::event::Event::AboutToWait => {
                }
                winit::event::Event::LoopExiting => {
                    target.exit();
                }
                _ => {},
            }
        });
    } // fn run
} // impl System

struct ParsedObj {
    pub vertices: Vec<render::Vertex>,
    pub indices: Vec<u32>,
}

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

fn camera_keyboard_control(camera: &mut render::Camera, input_state: &input::State, delta_time: f32) {
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
} // fn camera_keyboard_control

struct CameraController {

}

impl Unit for CameraController {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        camera_keyboard_control(state.render.lock_camera(), state.input_state, state.timer.get_delta_time());
    }
}

struct Light {
    light1: render::DirectionalLight,
    light2: render::DirectionalLight,
    light3: render::DirectionalLight,
}

impl Light {
    pub fn new(render: &mut render::Render) -> Light {
        Light {
            light1: render.create_directional_light(&render::DirectionalLightDescriptor {
                color: Vec3f::new(0.0, 1.0, 1.0),
                direction: -Vec3f::new(0.30, 0.47, 0.80),
                power: 0.0,
            }),
            light2: render.create_directional_light(&render::DirectionalLightDescriptor {
                color: Vec3f::new(1.0, 0.0, 0.0),
                direction: -Vec3f::new(0.30, 0.47, -0.80),
                power: 30.0,
            }),
            light3: render.create_directional_light(&render::DirectionalLightDescriptor {
                color: Vec3f::new(1.0, 1.0, 0.0),
                direction: -Vec3f::new(-0.30, 0.47, -0.80),
                power: 30.0,
            }),
        }
    }
}

impl Unit for Light {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        if !state.input_state.is_key_pressed(input::KeyCode::Digit1) {
            state.scene.add_directional_light(&mut self.light1);
        }
        if !state.input_state.is_key_pressed(input::KeyCode::Digit2) {
            state.scene.add_directional_light(&mut self.light2);
        }
        if !state.input_state.is_key_pressed(input::KeyCode::Digit3) {
            state.scene.add_directional_light(&mut self.light3);
        }
    }
}

struct Model {
    primitive: render::Primitive,
}

impl Model {
    fn new(render: &mut render::Render, data: &ParsedObj) -> Model {
        let mut primitive = render.create_primitive(&render::PrimitiveDescriptor {
            indices: Some(&data.indices),
            vertices: &data.vertices,
            material: &render::Material {
                base_color: Vec3f::new(1.00, 1.00, 1.00),
                metallic: 1.0,
                roughness: 0.5,
            }
        });

        primitive.lock_transforms().push(Mat4x4f::identity());

        Model { primitive }
    }
}

impl Unit for Model {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        state.scene.add_primitive(&mut self.primitive);
    }
}

struct Root {
    added: bool,
}

impl Root {
    pub fn new() -> Root {
        Root { added: false }
    }
}

impl Unit for Root {
    fn response<'a, 's, 'r>(&'a mut self, state: &mut SystemState<'s, 'a, 'r>) where 'r: 'a, 'a: 's {
        if !self.added {
            state.scene_context.add_unit(Box::new(CameraController{}));

            state.scene_context.add_unit(Box::new(Light::new(state.render)));
            state.scene_context.add_unit(Box::new(Model::new(state.render, &parse_obj(include_str!("../models/rei.obj")).unwrap())));
            state.scene_context.add_unit(Box::new(Model::new(state.render, &parse_obj(include_str!("../models/e1m1.obj")).unwrap())));
            state.scene_context.add_unit(Box::new(Model::new(state.render, &parse_obj(include_str!("../models/cow.obj")).unwrap())));

            self.added = true;
        }
    }
}

fn main() {
    let mut system = System::new().unwrap();

    system.get_scene().add_unit(Box::new(Root::new()));

    system.run();
} // fn main

// file main.rs