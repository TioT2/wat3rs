/// WAT3RS Project
/// `File` render/kernel.rs
/// `Description` Render kernel implementaiton module
/// `Author` TioT2
/// `Last changed` 17.02.2024

use std::sync::Arc;
use wgpu::core::device;

use crate::math::*;

/// Renderer most basic objects storage object representation structure
pub struct Kernel {
    /// WebGPU instance
    pub instance: wgpu::Instance,

    /// WebGPU adapter (physical video device representation)
    pub adapter: wgpu::Adapter,

    /// WebGPU device (main api-interaction object)
    pub device: wgpu::Device,

    /// WebGPU queue (execution queue)
    pub queue: wgpu::Queue,

    pub device_features: wgpu::Features,
} // struct Kernel

/// Window surface representation structure
pub struct Surface<'a> {
    kernel: Arc<Kernel>,
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
} // struct Surface

impl<'a> Surface<'a> {
    /// Surface reconfiguration function
    /// * `new_extent` - extent to reconfigure surface to
    pub fn resize(&mut self, new_extent: Vec2<usize>) {
        self.config.width = new_extent.x as u32;
        self.config.height = new_extent.y as u32;

        self.surface.configure(&self.kernel.device, &self.config);
    } // fn resize

    /// Current surface extent getting function
    /// * Returns current surface extent
    pub fn get_extent(&self) -> Vec2<usize> {
        Vec2 {
            x: self.config.width as usize,
            y: self.config.height as usize
        }
    } // fn get_extent

    /// Surface format getting function
    /// * Returns surface texture format
    pub fn get_format(&self) -> wgpu::TextureFormat {
        self.config.format
    } // fn get_format

    /// Current surface texture getting function
    /// * Returns current surface texture if it is
    pub fn get_texture(&mut self) -> Option<wgpu::SurfaceTexture> {
        self.surface.get_current_texture().map_or_else(
            |err| {
                match err {
                    wgpu::SurfaceError::Lost => {
                        let new_surface = self.kernel.instance.create_surface(self.window.clone()).unwrap();

                        if !self.kernel.adapter.is_surface_supported(&new_surface) {
                            return None;
                        }

                        self.surface = new_surface;
                    }
                    wgpu::SurfaceError::OutOfMemory => {
                        return None;
                    }
                    wgpu::SurfaceError::Outdated => {
                        let surface_size = self.window.inner_size();
                        self.config.width = surface_size.width;
                        self.config.height = surface_size.height;
                    }
                    wgpu::SurfaceError::Timeout => {
                        return None;
                    }
                }

                if self.config.width == 0 || self.config.height == 0 {
                    return None;
                }
                self.surface.configure(&self.kernel.device, &self.config);
                self.surface.get_current_texture().ok() // )))
            },
            |value| {
                Some(value)
            }
        )
    } // fn get_texture
} // impl Surface

impl Kernel {
    /// Kernel constructor
    /// * `window` - window to create surface basing on
    /// * Returns result, contained by newly created kernel and surface for `window` window.
    pub fn new<'a>(window: Arc<winit::window::Window>) -> Result<(Arc<Kernel>, Surface<'a>), String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: if cfg!(debug_assertions) { wgpu::Backends::GL } else { wgpu::Backends::PRIMARY },
            flags: if cfg!(debug_assertions) { wgpu::InstanceFlags::VALIDATION } else { wgpu::InstanceFlags::empty() },
            ..Default::default()
        });

        let surface: wgpu::Surface<'a> = instance.create_surface(window.clone()).map_err(|err| err.to_string())?;

        let adapter = futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })).ok_or("Error requesting fitting adapter".to_string())?;
        let device_features = adapter.features() & (
            wgpu::Features::POLYGON_MODE_LINE |
            wgpu::Features::POLYGON_MODE_POINT
        );

        let (device, queue) = futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor{
            required_features: device_features,
            ..Default::default()
        }, None)).map_err(|err| err.to_string())?;

        let surface_format = {
            let fmts = surface.get_capabilities(&adapter).formats;

            if fmts.contains(&wgpu::TextureFormat::Bgra8UnormSrgb) {
                wgpu::TextureFormat::Bgra8UnormSrgb
            } else {
                fmts[0]
            }
        };

        let surface_config = wgpu::SurfaceConfiguration {
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![surface_format],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let kernel_arc = Arc::new(Self {
            adapter,
            device,
            instance,
            queue,
            device_features,
        });

        Ok((
            kernel_arc.clone(),
            Surface {
                config: surface_config,
                kernel: kernel_arc.clone(),
                surface,
                window: window.clone(),
            }
        ))
    } // fn new
} // impl Kernel

// file kernel.rs
