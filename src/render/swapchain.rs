use std::sync::Arc;

use ash::{khr::surface, vk};

use super::kernel::Kernel;

pub struct Swapchain {
    kernel: Arc<Kernel>,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
}

pub enum SwapchainCreateError {
    VulkanError(vk::Result),
}

impl From<vk::Result> for SwapchainCreateError {
    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

impl Swapchain {
    pub fn new(kernel: Arc<Kernel>) -> Result<Self, SwapchainCreateError> {
        let present_mode = {
            let present_modes = unsafe {
                kernel
                    .instance_ext_surface
                    .get_physical_device_surface_present_modes(kernel.physical_device, kernel.surface)
                    ?
            };

            if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
                vk::PresentModeKHR::MAILBOX
            } else {
                vk::PresentModeKHR::FIFO
            }
        };
        let surface_format = {
            let surface_formats = unsafe {
                kernel
                    .instance_ext_surface
                    .get_physical_device_surface_formats(kernel.physical_device, kernel.surface)
                    ?
            };

            const WANTED_FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
                format: vk::Format::B8G8R8A8_SRGB
            };

            if surface_formats.contains(&WANTED_FORMAT) {
                WANTED_FORMAT
            } else {
                *surface_formats
                    .iter()
                    .find(|format| format.format == vk::Format::B8G8R8A8_SRGB)
                    .unwrap_or(&surface_formats[0])
            }
        };
        let surface_capabilities = unsafe {
            kernel
                .instance_ext_surface
                .get_physical_device_surface_capabilities(kernel.physical_device, kernel.surface)
                ?
        };

        let swapchain = unsafe {
            kernel
                .device_ext_swapchain
                .create_swapchain(
                    &vk::SwapchainCreateInfoKHR::default()
                        .image_format(surface_format.format)
                        .image_color_space(surface_format.color_space)
                        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                        .present_mode(present_mode)
                        .pre_transform(surface_capabilities.current_transform),
                    None
                )
                ?
        };

        let images = unsafe {
            kernel
                .device_ext_swapchain
                .get_swapchain_images(swapchain)
                ?
        };

        Ok(Swapchain {
            kernel,
            swapchain,
            images,
        })
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.kernel
                .device_ext_swapchain
                .destroy_swapchain(self.swapchain, None)
        }
    }
}