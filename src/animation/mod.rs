use std::sync::Arc;

pub mod timer;
pub mod input;
pub mod render;

pub struct State<'t> {
    pub timer : &'t mut timer::State<'t>,
    pub input : &'t mut input::State<'t>,
    pub render: &'t mut render::State<'t>,

    destroy_requested: bool,
    new_units: Vec<Box<dyn Unit>>,
}

impl<'t> State<'t> {
    /// Unit self-destroy requesting function
    pub fn request_self_remove(&mut self) {
        self.destroy_requested = true;
    } // fn request_self_remove

    /// Unit adding function
    /// * `new_unit` - unit to add to unit pool on next update
    pub fn add_unit(&mut self, new_unit: Box<dyn Unit>) {
        self.new_units.push(new_unit);
    } // fn add_unit
}

/// Actually, animation descriptor.
pub struct Animation {
    window_name: String,
    startup_unit: Box<dyn Unit>,
}

pub trait Unit {
    fn response(&mut self, state: &mut State);
}

pub struct DummyUnit {

}

impl Unit for DummyUnit {
    fn response(&mut self, _state: &mut State) {

    }
}

struct AnimationImpl {
    input: input::Input,
    timer: timer::Timer,
    render: render::Render,
    units: Vec<Box<dyn Unit>>,
    window: Arc<winit::window::Window>,
}

impl AnimationImpl {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop, descr: Box<Animation>) -> Option<Self> {
        let window = Arc::new(event_loop.create_window(
            winit::window::WindowAttributes::default()
                .with_title(descr.window_name)
        ).ok()?);

        Some(Self {
            input: input::Input::new(),
            timer: timer::Timer::new(),
            render: render::Render::new(window.clone())?,
            units: vec![descr.startup_unit],
            window,
        })
    }

    pub fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: winit::event::WindowEvent) {
        if window_id != self.window.id() {
            return;
        }

        match event {
            winit::event::WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                self.input.on_key_change(keycode, event.state == winit::event::ElementState::Pressed);
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit()
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.timer.response();

                let mut state = State {
                    destroy_requested: false,
                    new_units: Vec::new(),
                    input: &mut self.input.get_state(),
                    timer: &mut self.timer.get_state(),
                    render: &mut self.render.get_state(),
                };

                self.units = self.units
                    .drain(..)
                    .filter_map(|mut unit| {
                        unit.response(&mut state);

                        if state.destroy_requested {
                            state.destroy_requested = false;
                            None
                        } else {
                            Some(unit)
                        }
                    })
                    .collect();
                self.units.append(&mut state.new_units);

                self.render.render();
                self.input.clear_changed();

                self.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl Animation {
    pub fn new() -> Self {
        Self {
            startup_unit: Box::new(DummyUnit {}),
            window_name: "WAT3RS".into(),
        }
    } // fn new

    pub fn set_window_name(&mut self, name: &str) {
        self.window_name = name.into();
    } // fn set_window_name

    pub fn set_startup_unit(&mut self, startup_unit: Box<dyn Unit>) {
        self.startup_unit = startup_unit;
    } // fn set_startup_unit

    /// Animation starting function
    /// * Returns option that signs about it's success or fail
    pub fn run(self) -> Option<()> {
        struct Application {
            descriptor: Option<Box<Animation>>,
            anim: Option<AnimationImpl>,
        }

        impl winit::application::ApplicationHandler for Application {
            fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
                if self.descriptor.is_some() {
                    let mut descr = None;
                    std::mem::swap(&mut self.descriptor, &mut descr);
                    self.anim = AnimationImpl::new(event_loop, descr.unwrap());
                    self.descriptor = None;
                }
            }

            fn window_event(
                    &mut self,
                    event_loop: &winit::event_loop::ActiveEventLoop,
                    window_id: winit::window::WindowId,
                    event: winit::event::WindowEvent,
                ) {
                if let Some(anim) = self.anim.as_mut() {
                    anim.window_event(event_loop, window_id, event);
                }
            }
        }

        _ = winit::event_loop::EventLoop::new().ok()?.run_app(&mut Application {
            descriptor: Some(Box::new(self)),
            anim: None,
        });

        Some(())
    } // fn run
} // impl Animation

// file mod.rs
