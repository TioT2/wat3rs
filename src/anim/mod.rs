/// WAT3RS Project
/// `File` anim/mod.rs
/// `Description` Animation system main module
/// `Author` TioT2
/// `Last changed` 25.02.2024

use std::sync::Arc;

use crate::math::*;
use crate::render;
use crate::timer;
use crate::input;

pub struct SceneContext {
    added_units: Vec<Box<dyn Unit>>,
}

impl SceneContext {
    pub fn add_unit(&mut self, unit: Box<dyn Unit>) {
        self.added_units.push(unit);
    }
}

pub struct SystemState<'a, 'scene, 'render> where 'render: 'scene, 'scene: 'a {
    pub frame: &'a mut render::Frame<'scene>,
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
                                frame: &mut scene,
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

// file mod.rs
