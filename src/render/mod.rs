/// WAT3RS Project
/// `File` render/mod.rs
/// `Description` Main render implementation file
/// `Author` TioT2
/// `Last changed` 17.02.2024

pub use crate::math::*;

pub mod camera;

/// Vertex content represetnation structure
#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position : Vec3f,
    pub tex_coord: Vec2f,
    pub normal   : Vec3f,
}

/// Camera buffer content representation structure
#[repr(align(16))]
#[derive(Copy, Clone, Default)]
pub struct CameraBufferData {
    pub view_matrix: Mat4x4f,
    pub projection_matrix: Mat4x4f,
    pub view_projection_matrix: Mat4x4f,

    pub camera_location: Vec3f,
    _pad0: f32,
    pub camer_at: Vec3f,
    _pad1: f32,

    pub camera_direction: Vec3f,
    _pad2: f32,
    pub camera_right: Vec3f,
    _pad3: f32,
    pub camera_up: Vec3f,
    _pad4: f32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Material {
    pub base_color: Vec3f,
    pub roughness: f32,
    pub metallic: f32,
}

/// Primitive buffer data primitive index offset
const PRIMITIVE_BUFFER_DATA_PRIMITIVE_INDEX_OFFSET: usize = std::mem::size_of::<Vec3f>();

/// Per-primitive uniform buffer data representation structure
#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct PrimitiveBufferData {
    /// Primitive content index in global matrix pool
    pub base_color: Vec3f,
    pub primitive_index: u32,
    pub roughness: f32,
    pub metallic: f32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct WorldMatrixBufferElement {
    pub transform: Mat4x4f,
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
#[allow(unused)]
struct MatrixBufferElement {
    pub transform: Mat4x4f,
    pub world_inverse_0: Vec3f,
    pub world_inverse_1: Vec3f,
    pub world_inverse_2: Vec3f,
}

/// Primitive create info
pub struct PrimitiveDescriptor<'a> {
    pub vertices: &'a [Vertex],
    pub indices: Option<&'a [u32]>,
    pub material: &'a Material,
}

/// Rendering primitive representation structure
pub struct Primitive {
    // primitive bind group:
    // primitive uniform

    uniform_buffer: wgpu::Buffer,
    primitive_bind_group: wgpu::BindGroup,

    vertex_buffer: wgpu::Buffer, // primitive vertex and index buffer
    vertex_count: usize,

    index_buffer: wgpu::Buffer, // primitive vertex and index buffer
    index_count: usize,

    world_transforms: Vec<WorldMatrixBufferElement>,
}

impl Primitive {
    /// Lock all instances
    pub fn lock_transforms<'transform_lock>(&'transform_lock mut self) -> &'transform_lock mut Vec<WorldMatrixBufferElement> {
        &mut self.world_transforms
    }
}

struct Core<'a> {
    window: std::sync::Arc<winit::window::Window>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    surface: wgpu::Surface<'a>,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
}

impl<'a> Core<'a> {
    pub fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::VALIDATION,
            ..Default::default()
        });

        let surface: wgpu::Surface<'a> = instance.create_surface(window.clone()).map_err(|err| err.to_string())?;

        let adapter = futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })).ok_or("Error requesting fitting adapter".to_string())?;

        let (device, queue) = futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor{
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
            present_mode: wgpu::PresentMode::AutoVsync,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![surface_format],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            window,
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config,
        })
    }

    fn check_surface_compatiblity(&self, surface: &wgpu::Surface) -> bool {
        self.adapter.is_surface_supported(surface)
    }

    pub fn get_surface_texture(&mut self) -> Option<wgpu::SurfaceTexture> {
        self.surface.get_current_texture().map_or_else(
            |err| {
                match err {
                    wgpu::SurfaceError::Lost => {
                        let new_surface = self.instance.create_surface(self.window.clone()).unwrap();

                        if !self.check_surface_compatiblity(&new_surface) {
                            panic!("New surface isn't compatible");
                        }

                        self.surface = new_surface;
                    }
                    wgpu::SurfaceError::OutOfMemory => {
                        panic!("WebGPU is out of memory");
                    }
                    wgpu::SurfaceError::Outdated => {
                        let surface_size = self.window.inner_size();
                        self.surface_config.width = surface_size.width;
                        self.surface_config.height = surface_size.height;
                    }
                    wgpu::SurfaceError::Timeout => {
                        panic!("WebGPU frame request reached it's timeout!");
                    }
                }

                if self.surface_config.width == 0 || self.surface_config.height == 0 {
                    return None;
                }
                self.surface.configure(&self.device, &self.surface_config);
                self.surface.get_current_texture().ok() // )))
            },
            |value| {
                Some(value)
            }
        )
    }
}

struct Texture {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,

    extent: Vec2<usize>,
    format: wgpu::TextureFormat,
}

pub struct Render<'a> {
    core: Core<'a>,

    depth_texture: Texture,

    camera_buffer: wgpu::Buffer,
    world_matrix_buffer: wgpu::Buffer,
    matrix_buffer: wgpu::Buffer,
    matrix_capacity: usize,
    matrix_count: usize,

    primitive_pipeline: wgpu::RenderPipeline,
    primitive_pipeline_layout: wgpu::PipelineLayout,
    primitive_system_bind_group_layout: wgpu::BindGroupLayout,
    primitive_data_bind_group_layout: wgpu::BindGroupLayout,
    primitive_system_bind_group: wgpu::BindGroup,

    matrix_pipeline: wgpu::ComputePipeline,
    matrix_pipeline_layout: wgpu::PipelineLayout,
    matrix_bind_group_layout: wgpu::BindGroupLayout,
    matrix_bind_group: wgpu::BindGroup,

    camera: camera::Camera,
}

impl<'a> Render<'a> {
    /// Matrix buffers create function
    /// * `core` - render core
    /// * `capacity` - matrix buffers capacity
    /// * Returns (world matrix buffer, matrix buffer) tuple.
    fn create_matrix_buffers(core: &Core, capacity: usize) -> (wgpu::Buffer, wgpu::Buffer) {
        return (
            core.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: (capacity * std::mem::size_of::<WorldMatrixBufferElement>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            core.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: (capacity * std::mem::size_of::<MatrixBufferElement>()) as u64,
                usage: wgpu::BufferUsages::STORAGE,
            }),
        );
    }

    /// System bind groups create function
    /// * `core` - render core
    /// * `primitive_system_bind_group_layout` - first bind group layout
    /// * `matrix_bind_group_layout` - second bind group layout
    /// * `camera_buffer` - camera buffer
    /// * `world_matrix_buffer` - buffer for with world matrix to be placed in
    /// * `matrix_buffer` - buffer result matrix will be placed in
    /// * Returns (primitive_system_bind_group, matix_bind_group)
    fn build_bind_groups(core: &Core,
        primitive_system_bind_group_layout: &wgpu::BindGroupLayout,
        matrix_bind_group_layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
        world_matrix_buffer: &wgpu::Buffer,
        matrix_buffer: &wgpu::Buffer
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        (
            core.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &primitive_system_bind_group_layout,
                entries: &[
                    /* Camera UBO */
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &camera_buffer,
                        offset: 0,
                        size: None,
                    })},
                    /* World matrix SSBO */
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &world_matrix_buffer,
                        offset: 0,
                        size: None,
                    })},
                    /* Matrix SSBO */
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &matrix_buffer,
                        offset: 0,
                        size: None,
                    })},
                ]
            }),
            core.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &matrix_bind_group_layout,
                entries: &[
                    /* Camera buffer */
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &camera_buffer,
                        offset: 0,
                        size: None,
                    })},
                    /* World matrix SSBO */
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &world_matrix_buffer,
                        offset: 0,
                        size: None,
                    })},
                    /* Matrix SSBO */
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &matrix_buffer,
                        offset: 0,
                        size: None,
                    })},
                ],
            })
        )
    }

    pub fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Render<'a>, String> {
        let core = Core::new(window)?;

        // Initialize render target
        let depth_texture = {
            let depth_format = wgpu::TextureFormat::Depth24Plus;

            let texture = core.device.create_texture(&wgpu::TextureDescriptor {
                dimension: wgpu::TextureDimension::D2,
                format: depth_format,
                mip_level_count: 1,
                sample_count: 1,
                size: wgpu::Extent3d {width: core.surface_config.width, height: core.surface_config.height, depth_or_array_layers: 1},
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[depth_format],
                label: None,
            });

            Texture {
                extent: Vec2::<usize>::new(core.surface_config.width as usize, core.surface_config.height as usize),
                format: wgpu::TextureFormat::Depth24Plus,
                texture_view: texture.create_view(&wgpu::TextureViewDescriptor { ..Default::default() }),
                texture,
            }
        };

        let primitive_shader = core.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("primitive.wgsl"))),
        });
        let matrix_shader = core.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("matrix.wgsl")))
        });

        let matrix_capacity = 32usize;
        let (world_matrix_buffer, matrix_buffer) = Self::create_matrix_buffers(&core, matrix_capacity);

        let camera_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<CameraBufferData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });


        let primitive_data_bind_group_layout = core.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                /* Primitive offset and material UBO */
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<PrimitiveBufferData>() as u64).unwrap())
                    }
                },
            ],
        });

        let primitive_system_bind_group_layout = core.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                /* Camera UBO */
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<CameraBufferData>() as u64).unwrap())
                    },
                },
                /* World matrix SSBO */
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<WorldMatrixBufferElement>() as u64).unwrap()),
                    }
                },
                /* Matrix SSBO */
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<MatrixBufferElement>() as u64).unwrap()),
                    }
                }
            ],
        });

        let primitive_pipeline_layout = core.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&primitive_system_bind_group_layout, &primitive_data_bind_group_layout],
            push_constant_ranges: &[],
        });

        let primitive_pipeline = core.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&primitive_pipeline_layout),

            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState {..Default::default()},
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                format: depth_texture.format,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0
                }
            }),
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &primitive_shader,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: None,
                    format: core.surface_config.format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multisample: Default::default(),
            multiview: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: None,
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                entry_point: "vs_main",
                module: &primitive_shader,
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        /* position */
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        /* tex_coord */
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: std::mem::size_of::<Vec3f>() as u64,
                            shader_location: 1,
                        },
                        /* normal */
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::size_of::<Vec3f>() as u64 + std::mem::size_of::<Vec2f>() as u64,
                            shader_location: 2,
                        },
                    ],
                }],
            },
        });

        let matrix_bind_group_layout = core.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                /* Camera buffer */
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<CameraBufferData>() as u64).unwrap()),
                    },
                },
                /* World matrix buffer */
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<Mat4x4f>() as u64).unwrap()),
                    },
                },
                /* Matrix buffer */
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<Mat4x4f>() as u64 * 2).unwrap()),
                    },
                },
            ]
        });

        let matrix_pipeline_layout = core.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&matrix_bind_group_layout],
        });

        let matrix_pipeline = core.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            entry_point: "cs_main",
            layout: Some(&matrix_pipeline_layout),
            module: &matrix_shader,
        });

        let (primitive_system_bind_group, matrix_bind_group) = Self::build_bind_groups(
            &core,
            &primitive_system_bind_group_layout,
            &matrix_bind_group_layout,
            &camera_buffer,
            &world_matrix_buffer,
            &matrix_buffer
        );

        /* RENDER INITIALIZATION, MAZAFAKA */
        Ok(Self {
            core,
            camera_buffer,
            depth_texture,
            matrix_bind_group,
            matrix_bind_group_layout,
            matrix_buffer,
            matrix_capacity,
            matrix_count: 0usize,
            matrix_pipeline,
            matrix_pipeline_layout,
            primitive_data_bind_group_layout,
            primitive_pipeline,
            primitive_pipeline_layout,
            primitive_system_bind_group,
            primitive_system_bind_group_layout,
            world_matrix_buffer,
            camera: camera::Camera::new(),
        })
    }

    pub fn lock_camera<'b>(&'b mut self) -> &'b mut camera::Camera {
        &mut self.camera
    }

    pub fn update_camera_buffer(&mut self) {
        let loc = self.camera.get_location();
        let matrices = self.camera.get_matrices();

        self.core.queue.write_buffer(&self.camera_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(&CameraBufferData {
                view_matrix: matrices.view,
                projection_matrix: matrices.projection,
                view_projection_matrix: matrices.view_projection,
                camera_location: loc.location,
                camer_at: loc.at,

                camera_direction: loc.direction,
                camera_right: loc.right,
                camera_up: loc.up,
                ..Default::default()
            }), std::mem::size_of::<CameraBufferData>())
        });
    }

    /// Primitive create function
    /// * `descriptor` - created primitive descriptor
    /// * Returns created primitive
    pub fn create_primitive(&mut self, descriptor: &PrimitiveDescriptor) -> Primitive {
        let uniform_buffer = self.core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<PrimitiveBufferData>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        self.core.queue.write_buffer(&uniform_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(&PrimitiveBufferData {
                base_color: descriptor.material.base_color,
                primitive_index: 0,
                metallic: descriptor.material.metallic,
                roughness: descriptor.material.roughness,
            }), std::mem::size_of::<PrimitiveBufferData>())
        });

        let primitive_bind_group = self.core.device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<PrimitiveBufferData>() as u64).unwrap()),
                })
            }],
            label: None,
            layout: &self.primitive_data_bind_group_layout,
        });

        let vertex_buffer = self.core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: (std::mem::size_of::<Vertex>() * descriptor.vertices.len()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        self.core.queue.write_buffer(&vertex_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(descriptor.vertices.as_ptr()), descriptor.vertices.len() * std::mem::size_of::<Vertex>())
        });

        let index_buffer = self.core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: descriptor.indices.map_or(0, |idx| (std::mem::size_of::<u32>() * idx.len()) as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });
        _ = descriptor.indices.map_or((), |idx| {
            self.core.queue.write_buffer(&index_buffer, 0, unsafe {
                std::slice::from_raw_parts(std::mem::transmute(idx.as_ptr()), idx.len() * std::mem::size_of::<u32>())
            });
        });

        Primitive {
            index_buffer,
            index_count: descriptor.indices.map_or(0, |idx| idx.len()),
            primitive_bind_group,
            uniform_buffer,
            vertex_buffer,
            vertex_count: descriptor.vertices.len(),
            world_transforms: Vec::new(),
        }
    }

    /// Scene create function
    /// * Returns created scene
    pub fn create_scene<'b>(&mut self) -> Scene<'b> {
        Scene {
            primitives: Vec::new(),
        }
    }

    /// Scene rendering and presentation requesting function
    /// * `scene` - scene to render
    pub fn render_scene(&mut self, scene: &Scene) {
        let surface_texture = match self.core.get_surface_texture() {
            Some(tex) => tex,
            None => return
        };
        let surface_texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor { ..Default::default() });

        let required_matrix_capacity = {
            let mut sum = 0;
            for primitive in &scene.primitives {
                sum += primitive.world_transforms.len();
            }
            sum
        };

        // Fix matrix capacity if needed
        if self.matrix_capacity < required_matrix_capacity {
            self.matrix_capacity = required_matrix_capacity;
            self.matrix_count = required_matrix_capacity;
            // Update buffers and write them
            (self.world_matrix_buffer, self.matrix_buffer) = Self::create_matrix_buffers(&self.core, self.matrix_capacity);
            (self.primitive_system_bind_group, self.matrix_bind_group) = Self::build_bind_groups(&self.core, &self.primitive_system_bind_group_layout, &self.matrix_bind_group_layout, &self.camera_buffer, &self.world_matrix_buffer, &self.matrix_buffer);
        }

        // rewrite offsets and matrices
        {
            let mut offset: usize = 0;

            for primitive in &scene.primitives {
                // Write index into uniform buffer
                self.core.queue.write_buffer(&primitive.uniform_buffer, PRIMITIVE_BUFFER_DATA_PRIMITIVE_INDEX_OFFSET as u64, &(offset as u32).to_le_bytes());

                // Write matrices into world matrix buffer
                self.core.queue.write_buffer(&self.world_matrix_buffer, (offset * std::mem::size_of::<WorldMatrixBufferElement>()) as u64, unsafe {
                    std::slice::from_raw_parts(
                        std::mem::transmute(primitive.world_transforms.as_ptr()),
                        primitive.world_transforms.len() * std::mem::size_of::<WorldMatrixBufferElement>()
                    )
                });
                offset += primitive.world_transforms.len();
            }

            self.matrix_count = offset;
        }

        self.update_camera_buffer();

        {
            let mut command_encoder = self.core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { ..Default::default() });

            // Recalculate world-view-projection matrices
            {
                let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { ..Default::default() });
                compute_pass.set_bind_group(0, &self.matrix_bind_group, &[]);
                compute_pass.set_pipeline(&self.matrix_pipeline);
                compute_pass.dispatch_workgroups(self.matrix_count as u32, 1, 1);
            }

            // Perform render pass
            {
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.30, g: 0.47, b: 0.80, a: 0.00 }),
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                            view: &surface_texture_view,
                        })
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        depth_ops: Some(wgpu::Operations::<f32> {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                        view: &self.depth_texture.texture_view,
                    }),
                    ..Default::default()
                });
                render_pass.set_viewport(0.0, 0.0, self.core.surface_config.width as f32, self.core.surface_config.height as f32, 0.0, 1.0);

                render_pass.set_pipeline(&self.primitive_pipeline);
                render_pass.set_bind_group(0, &self.primitive_system_bind_group, &[]);
                for primitive in &scene.primitives {
                    render_pass.set_bind_group(1, &primitive.primitive_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, primitive.vertex_buffer.slice(0..));
                    if primitive.index_count == 0 {
                        render_pass.draw(0..(primitive.vertex_count as u32), 0..(primitive.world_transforms.len() as u32));
                    } else {
                        render_pass.set_index_buffer(primitive.index_buffer.slice(0..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..(primitive.index_count as u32), 0, 0..(primitive.world_transforms.len() as u32));
                    }
                }
            }

            let command_buffer = command_encoder.finish();

            self.core.queue.submit([command_buffer]);
            surface_texture.present();
        }
    }
}

// To-display primitive collection
pub struct Scene<'a> {
    primitives: Vec<&'a Primitive>,
}

impl<'a> Scene<'a> {
    pub fn draw_primitive(&mut self, primitive: &'a Primitive) {
        self.primitives.push(primitive);
    }
}
