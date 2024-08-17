use std::sync::Arc;

struct Context {
    window: Arc<winit::window::Window>,
}

struct ApplicationHandler {
    context: Option<Context>,
}

impl ApplicationHandler {
    pub fn new() -> Self {
        Self { context: None }
    }
}

impl winit::application::ApplicationHandler for ApplicationHandler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title("WAT3RS")
                        .with_resizable(false),
                )
                .expect("Error creating window"),
        );

        self.context = Some(Context { window });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        type Event = winit::event::WindowEvent;

        if event == Event::CloseRequested {
            event_loop.exit();
            return;
        }

        let Some(context) = self.context.as_mut() else {
            return;
        };

        if context.window.id() != window_id {
            return;
        }

        match event {
            _ => {}
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("Error setting up event loop");

    event_loop
        .run_app(&mut ApplicationHandler::new())
        .expect("Error starting application");
}
