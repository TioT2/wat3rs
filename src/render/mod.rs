use std::sync::Arc;

use kernel::Kernel;

pub(self) mod kernel;
pub(self) mod queue_family_indices;
pub(self) mod swapchain;

pub struct Render {
    kernel: Arc<Kernel>,
}

pub enum RenderCreateError {
    Unknown,
}

impl Render {
    pub fn new(
        window_handle: raw_window_handle::RawWindowHandle,
        display_handle: raw_window_handle::RawDisplayHandle,
    ) -> Result<Self, RenderCreateError> {
        let kernel =
            Arc::new(Kernel::new(window_handle, display_handle).expect("Error creating kernel"));

        Ok(Self { kernel })
    }
}
