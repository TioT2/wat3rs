/// WAT3RS Project
/// `File` main.rs
/// `Description` Main project file
/// `Author` TioT2
/// `Last changed` 17.02.2024

pub mod math;

pub mod render;
pub mod timer;
pub mod input;

use std::io::Read;

use math::*;

struct ParsedObj {
    pub vertices: Vec<render::Vertex>,
    pub indices: Vec<u32>,
}

fn load_obj(path: &str) -> Result<ParsedObj, String> {
    let source = {
        let mut file = std::fs::File::open(path).map_err(|err| err.to_string())?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(|err| err.to_string())?;
        buffer
    };

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
        (input_state.is_key_pressed(winit::keyboard::KeyCode::KeyD) as i32 - input_state.is_key_pressed(winit::keyboard::KeyCode::KeyA) as i32) as f32,
        (input_state.is_key_pressed(winit::keyboard::KeyCode::KeyR) as i32 - input_state.is_key_pressed(winit::keyboard::KeyCode::KeyF) as i32) as f32,
        (input_state.is_key_pressed(winit::keyboard::KeyCode::KeyW) as i32 - input_state.is_key_pressed(winit::keyboard::KeyCode::KeyS) as i32) as f32,
    );
    let rotate_axis = Vec2f::new(
      (input_state.is_key_pressed(winit::keyboard::KeyCode::ArrowRight) as i32 - input_state.is_key_pressed(winit::keyboard::KeyCode::ArrowLeft) as i32) as f32,
      (input_state.is_key_pressed(winit::keyboard::KeyCode::ArrowDown) as i32 - input_state.is_key_pressed(winit::keyboard::KeyCode::ArrowUp) as i32) as f32,
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

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let window = std::sync::Arc::new(winit::window::WindowBuilder::new()
        .with_title("wat3rs")
        // .with_resizable(false)
        .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(800, 600)))
        .build(&event_loop).unwrap());

    let mut render = render::Render::new(window.clone()).unwrap();
    let mut timer = timer::Timer::new();
    let mut input = input::Input::new();

    let mut model = {
        let model_obj = load_obj("models/e1m1.obj").unwrap();

        render.create_primitive(&render::PrimitiveDescriptor {
            indices: Some(&model_obj.indices),
            vertices: &model_obj.vertices,
            material: &render::Material {
                base_color: Vec3f::new(1.0, 1.0, 1.0),
                roughness: 0.5,
                metallic: 0.5,
            },
        })
    };

    model.lock_transforms().push(render::WorldMatrixBufferElement { transform: math::Mat4x4f::identity() });

    let mut triangle = render.create_primitive(&render::PrimitiveDescriptor {
        indices: None,
        vertices: &[
            render::Vertex {
                position: math::Vec3f::new(0.0, 1.0, 0.0),
                tex_coord: math::Vec2f::new(0.0, 0.0),
                normal: math::Vec3f::new(0.0, 0.0, 1.0),
            },
            render::Vertex {
                position: math::Vec3f::new(-0.866, -0.5, 0.0),
                tex_coord: math::Vec2f::new(0.0, 0.0),
                normal: math::Vec3f::new(0.0, 0.0, 1.0),
            },
            render::Vertex {
                position: math::Vec3f::new(0.866, -0.5, 0.0),
                tex_coord: math::Vec2f::new(0.0, 0.0),
                normal: math::Vec3f::new(0.0, 0.0, 1.0),
            },
        ],
        material: &render::Material {
            base_color: math::Vec3f::new(0.0, 1.0, 0.0),
            metallic: 0.1,
            roughness: 0.1,
        }
    });

    triangle.lock_transforms().push(render::WorldMatrixBufferElement { transform: math::Mat4x4f::identity() });

    let mut pressed_key_codes = Vec::<winit::keyboard::KeyCode>::new();
    let mut frame_index = 0;

    event_loop.run(move |event, target| {

        match event {
            winit::event::Event::WindowEvent { window_id, event } => if window.id() == window_id {
                match event {
                    winit::event::WindowEvent::CloseRequested => target.exit(),
                    winit::event::WindowEvent::KeyboardInput { event, .. } => if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        match event.state {
                            winit::event::ElementState::Pressed => input.on_key_state_change(code, input::KeyState::Pressed),
                            winit::event::ElementState::Released => input.on_key_state_change(code, input::KeyState::Released),
                        }
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        timer.response();
                        if timer.get_fps() > 1.0 {
                            if frame_index % (timer.get_fps().ceil() as u32) == 1 {
                                println!("FPS: {}", timer.get_fps());
                            }
                        } else {
                            println!("Less, than 1 FPS");
                        }
                        frame_index += 1;

                        // {
                        //     let transforms = triangle.lock_transforms();
                        //     let mat = render::WorldMatrixBufferElement { transform: math::Mat4x4f::rotate_y(timer.get_time()) };
                        //     if transforms.is_empty() {
                        //         transforms.push(mat);
                        //     } else {
                        //         transforms[0] = mat;
                        //     }
                        // }

                        let mut scene = render.create_scene();
                        scene.draw_primitive(&model);
                        scene.draw_primitive(&triangle);

                        camera_keyboard_control(render.lock_camera(), input.get_state(), timer.get_delta_time());
                        pressed_key_codes.clear();
                        render.render_scene(&scene);
                        window.request_redraw();
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
    }).unwrap();
}
