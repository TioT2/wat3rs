/// WAT3RS Project
/// `File` render/mod.rs
/// `Description` Main render implementation module
/// `Author` TioT2
/// `Last changed` 17.02.2024

mod camera;
mod kernel;
mod texture;

pub use camera::*;
use kernel::*;
pub use texture::*;

use std::sync::Arc;

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

pub type WorldMatrixBufferElement = Mat4x4f;

// #[repr(C, align(16))]
// #[derive(Copy, Clone)]
// pub struct WorldMatrixBufferElement {
//     pub transform: Mat4x4f,
// }

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
    uniform_buffer: wgpu::Buffer,
    primitive_bind_group: wgpu::BindGroup,

    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,

    index_buffer: wgpu::Buffer,
    index_count: usize,

    world_transforms: Vec<WorldMatrixBufferElement>,
} // struct Primitive

impl Primitive {
    /// Lock all instances
    /// * Returns mutable reference to world matrix buffer
    pub fn lock_transforms<'transform_lock>(&'transform_lock mut self) -> &'transform_lock mut Vec<WorldMatrixBufferElement> {
        &mut self.world_transforms
    } // fn lock_transforms
} // impl Primitive

pub struct Target {
    position_id: Texture,
    normal_id: Texture,
    base_color_opcaity: Texture,
    metallic_roughness_occlusion_meta: Texture,
    depth: Texture,
}

impl Kernel {
    /// Matrix buffers create function
    /// * `capacity` - matrix buffers capacity
    /// * Returns (world matrix buffer, matrix buffer) tuple.
    fn create_matrix_buffers(&self, capacity: usize) -> (wgpu::Buffer, wgpu::Buffer) {
        return (
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: (capacity * std::mem::size_of::<WorldMatrixBufferElement>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: (capacity * std::mem::size_of::<MatrixBufferElement>()) as u64,
                usage: wgpu::BufferUsages::STORAGE,
            }),
        );
    } // fn create_matrix_buffers

    /// System bind groups create function
    /// * `primitive_system_bind_group_layout` - first bind group layout
    /// * `matrix_bind_group_layout` - second bind group layout
    /// * `camera_buffer` - camera buffer
    /// * `world_matrix_buffer` - buffer for with world matrix to be placed in
    /// * `matrix_buffer` - buffer result matrix will be placed in
    /// * Returns (primitive_system_bind_group, matix_bind_group)
    fn build_bind_groups(&self,
        primitive_system_bind_group_layout: &wgpu::BindGroupLayout,
        matrix_bind_group_layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
        world_matrix_buffer: &wgpu::Buffer,
        matrix_buffer: &wgpu::Buffer
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        (
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
    } // fn build_bind_groups

    /// Target create function
    /// * `extent` - target extent
    /// * Returns created target
    fn create_target(&self, extent: Vec2<usize>) -> Target {
        let extent = wgpu::Extent3d {
            width: extent.x as u32,
            height: extent.y as u32,
            depth_or_array_layers: 1,
        };
        let descriptor = wgpu::TextureDescriptor {
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: extent,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba32Float]
        };
        Target {
            position_id: self.create_texture(&wgpu::TextureDescriptor {
                format: wgpu::TextureFormat::Rgba32Float,
                view_formats: &[wgpu::TextureFormat::Rgba32Float],
                ..descriptor
            }),
            normal_id: self.create_texture(&wgpu::TextureDescriptor {
                format: wgpu::TextureFormat::Rgba16Sint,
                view_formats: &[wgpu::TextureFormat::Rgba16Sint],
                ..descriptor
            }),
            base_color_opcaity: self.create_texture(&wgpu::TextureDescriptor {
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                ..descriptor
            }),
            metallic_roughness_occlusion_meta: self.create_texture(&wgpu::TextureDescriptor {
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                ..descriptor
            }),
            depth: self.create_texture(&wgpu::TextureDescriptor {
                format: wgpu::TextureFormat::Depth24Plus,
                view_formats: &[wgpu::TextureFormat::Depth24Plus],
                ..descriptor
            })
        }
    } // fn create_target

    /// Target bind group create function
    /// * Returns target sampler
    fn create_target_bind_group(&self, target: &Target, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(target.position_id.get_view()) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(target.normal_id.get_view()) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(target.base_color_opcaity.get_view()) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(target.metallic_roughness_occlusion_meta.get_view()) },
            ]
        })
    } // fn create_target_bind_group
}

pub struct DirectionalLight {
    kernel: Arc<Kernel>,
    data_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    data: DirectionalLightBufferData,
}

/// Directional light params representation structure
#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct DirectionalLightBufferData {
    pub direction: Vec3f,
    pub power: f32,
    pub color: Vec3f,
} // struct DirectionalLightData

/// Directional light descritpor
pub type DirectionalLightDescriptor = DirectionalLightBufferData;

/// Directional light content
pub type DirectionalLightData = DirectionalLightBufferData;

impl DirectionalLight {
    pub fn get_data(&self) -> DirectionalLightData {
        self.data
    } // fn get_data

    pub fn set_data(&mut self, directional_light_data: &DirectionalLightData) {
        self.data = *directional_light_data;
        self.kernel.queue.write_buffer(&self.data_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(directional_light_data), std::mem::size_of::<DirectionalLightBufferData>())
        })
    } // fn set_data
} // impl DirectionalLight

/// Renderer representation structure
pub struct Render<'a> {
    kernel: Arc<Kernel>,
    surface: Surface<'a>,

    target: Target,

    target_bind_group_layout: wgpu::BindGroupLayout,
    target_bind_group: wgpu::BindGroup,

    directional_light_pipeline: wgpu::RenderPipeline,
    directional_light_system_bind_group: wgpu::BindGroup,
    directional_light_data_bind_group_layout: wgpu::BindGroupLayout,

    camera_buffer: wgpu::Buffer,
    world_matrix_buffer: wgpu::Buffer,
    matrix_buffer: wgpu::Buffer,
    matrix_capacity: usize,
    matrix_count: usize,

    primitive_pipeline: wgpu::RenderPipeline,
    primitive_system_bind_group_layout: wgpu::BindGroupLayout,
    primitive_data_bind_group_layout: wgpu::BindGroupLayout,
    primitive_system_bind_group: wgpu::BindGroup,

    matrix_pipeline: wgpu::ComputePipeline,
    matrix_bind_group_layout: wgpu::BindGroupLayout,
    matrix_bind_group: wgpu::BindGroup,

    camera: Camera,
} // struct Render<'a>

impl<'a> Render<'a> {
    pub fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Render<'a>, String> {
        let (kernel, surface) = Kernel::new(window)?;

        let target = kernel.create_target(surface.get_extent());

        let primitive_shader = kernel.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/primitive.wgsl"))),
        });
        let matrix_shader = kernel.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/matrix.wgsl")))
        });

        let matrix_capacity = 32usize;
        let (world_matrix_buffer, matrix_buffer) = kernel.create_matrix_buffers(matrix_capacity);

        let camera_buffer = kernel.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<CameraBufferData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });


        let primitive_data_bind_group_layout = kernel.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let primitive_system_bind_group_layout = kernel.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let target_bind_group_layout = kernel.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                /* position_id */
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }
                },
                /* normal_id */
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Sint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }
                },
                /* color_opcaity */
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }
                },
                /* metallic_roughness_occlusion_meta */
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }
                },
            ]
        });
        let target_bind_group = kernel.create_target_bind_group(&target, &target_bind_group_layout);

        let directional_light_system_bind_group_layout = kernel.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<CameraBufferData>() as u64).unwrap()),
                    }
                }
            ]
        });

        let directional_light_system_bind_group = kernel.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &directional_light_system_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &camera_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<CameraBufferData>() as u64).unwrap())
                    })
                }
            ]
        });

        let directional_light_data_bind_group_layout = kernel.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<DirectionalLightBufferData>() as u64).unwrap()),
                    }
                }
            ]
        });

        let directional_light_pipeline_layout = kernel.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&target_bind_group_layout, &directional_light_system_bind_group_layout, &directional_light_data_bind_group_layout],
            ..Default::default()
        });

        let directional_light_shader = kernel.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/directional_light.wgsl")))
        });

        let directional_light_pipeline = kernel.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &directional_light_shader,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent {
                            dst_factor: wgpu::BlendFactor::One,
                            src_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    format: surface.get_format(),
                    write_mask: wgpu::ColorWrites::all(),
                })]
            }),
            vertex: wgpu::VertexState {
                buffers: &[],
                entry_point: "vs_main",
                module: &directional_light_shader,
            },
            label: None,
            layout: Some(&directional_light_pipeline_layout),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                unclipped_depth: false,
            },
        });

        let primitive_pipeline_layout = kernel.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&primitive_system_bind_group_layout, &primitive_data_bind_group_layout],
            push_constant_ranges: &[],
        });

        let primitive_pipeline = kernel.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&primitive_pipeline_layout),

            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState {..Default::default()},
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                format: target.depth.get_texture().format(),
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
                targets: &[
                    /* position_id */
                    Some(wgpu::ColorTargetState {
                        blend: None,
                        format: target.position_id.get_texture().format(),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    /* normal_instance */
                    Some(wgpu::ColorTargetState {
                        blend: None,
                        format: target.normal_id.get_texture().format(),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    /* base_color_opcaity */
                    Some(wgpu::ColorTargetState {
                        blend: None,
                        format: target.base_color_opcaity.get_texture().format(),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    /* metallic_roughness_occlusion_meta */
                    Some(wgpu::ColorTargetState {
                        blend: None,
                        format: target.metallic_roughness_occlusion_meta.get_texture().format(),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            multisample: Default::default(),
            multiview: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: Some(wgpu::Face::Back),
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

        let matrix_bind_group_layout = kernel.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let matrix_pipeline_layout = kernel.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&matrix_bind_group_layout],
        });

        let matrix_pipeline = kernel.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            entry_point: "cs_main",
            layout: Some(&matrix_pipeline_layout),
            module: &matrix_shader,
        });

        let (primitive_system_bind_group, matrix_bind_group) = kernel.build_bind_groups(
            &primitive_system_bind_group_layout,
            &matrix_bind_group_layout,
            &camera_buffer,
            &world_matrix_buffer,
            &matrix_buffer
        );

        /* RENDER INITIALIZATION, MAZAFAKA */
        Ok(Self {
            kernel,
            surface,
            camera_buffer,
            target,
            target_bind_group_layout,
            target_bind_group,
            directional_light_data_bind_group_layout,
            directional_light_system_bind_group,
            directional_light_pipeline,
            matrix_bind_group,
            matrix_bind_group_layout,
            matrix_buffer,
            matrix_capacity,
            matrix_count: 0usize,
            matrix_pipeline,
            primitive_data_bind_group_layout,
            primitive_pipeline,
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

        self.kernel.queue.write_buffer(&self.camera_buffer, 0, unsafe {
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

    /// Directional light create funciton
    /// * `descriptor` - direcitonal light descriptor
    /// * Returns created directional light
    pub fn create_directional_light(&mut self, descriptor: &DirectionalLightDescriptor) -> DirectionalLight {
        let data_buffer = self.kernel.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<DirectionalLightBufferData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        self.kernel.queue.write_buffer(&data_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(descriptor), std::mem::size_of::<DirectionalLightBufferData>())
        });

        let bind_group = self.kernel.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.directional_light_data_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        offset: 0,
                        buffer: &data_buffer,
                        size: Some(std::num::NonZeroU64::try_from(std::mem::size_of::<DirectionalLightBufferData>() as u64).unwrap()),
                    })
                }
            ]
        });

        DirectionalLight {
            kernel: self.kernel.clone(),
            data: *descriptor,
            data_buffer,
            bind_group,
        }
    } // fn create_directional_light

    /// Primitive create function
    /// * `descriptor` - created primitive descriptor
    /// * Returns created primitive
    pub fn create_primitive(&mut self, descriptor: &PrimitiveDescriptor) -> Primitive {
        let uniform_buffer = self.kernel.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<PrimitiveBufferData>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        self.kernel.queue.write_buffer(&uniform_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(&PrimitiveBufferData {
                base_color: descriptor.material.base_color,
                primitive_index: 0,
                metallic: descriptor.material.metallic,
                roughness: descriptor.material.roughness,
            }), std::mem::size_of::<PrimitiveBufferData>())
        });

        let primitive_bind_group = self.kernel.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let vertex_buffer = self.kernel.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: (std::mem::size_of::<Vertex>() * descriptor.vertices.len()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        self.kernel.queue.write_buffer(&vertex_buffer, 0, unsafe {
            std::slice::from_raw_parts(std::mem::transmute(descriptor.vertices.as_ptr()), descriptor.vertices.len() * std::mem::size_of::<Vertex>())
        });

        let index_buffer = self.kernel.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: descriptor.indices.map_or(0, |idx| (std::mem::size_of::<u32>() * idx.len()) as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });
        _ = descriptor.indices.map_or((), |idx| {
            self.kernel.queue.write_buffer(&index_buffer, 0, unsafe {
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
    } // fn create_primitive

    /// Scene create function
    /// * Returns created scene
    pub fn create_scene<'b>(&mut self) -> Scene<'b> {
        Scene {
            primitives: Vec::new(),
            directional_lights: Vec::new(),
        }
    } // fn create_scene

    /// Renderer resize function
    /// * `new_extent` - new renderer extent
    pub fn resize(&mut self, new_extent: Vec2<usize>) {
        self.target = self.kernel.create_target(new_extent);
        self.target_bind_group = self.kernel.create_target_bind_group(&self.target, &self.target_bind_group_layout);
        self.surface.resize(new_extent);

        self.camera.resize(new_extent);
    } // fn resize

    /// Scene rendering and presentation requesting function
    /// * `scene` - scene to render
    pub fn render_scene(&mut self, scene: &Scene) {
        let surface_texture = match self.surface.get_texture() {
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
            (self.world_matrix_buffer, self.matrix_buffer) = self.kernel.create_matrix_buffers(self.matrix_capacity);
            (self.primitive_system_bind_group, self.matrix_bind_group) = self.kernel.build_bind_groups(&self.primitive_system_bind_group_layout, &self.matrix_bind_group_layout, &self.camera_buffer, &self.world_matrix_buffer, &self.matrix_buffer);
        }

        // rewrite offsets and matrices
        {
            let mut offset: usize = 0;

            for primitive in &scene.primitives {
                // Write index into uniform buffer
                self.kernel.queue.write_buffer(&primitive.uniform_buffer, PRIMITIVE_BUFFER_DATA_PRIMITIVE_INDEX_OFFSET as u64, &(offset as u32).to_le_bytes());

                // Write matrices into world matrix buffer
                self.kernel.queue.write_buffer(&self.world_matrix_buffer, (offset * std::mem::size_of::<WorldMatrixBufferElement>()) as u64, unsafe {
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
            let mut command_encoder = self.kernel.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { ..Default::default() });

            // Recalculate world-view-projection matrices
            {
                let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { ..Default::default() });
                compute_pass.set_bind_group(0, &self.matrix_bind_group, &[]);
                compute_pass.set_pipeline(&self.matrix_pipeline);
                compute_pass.dispatch_workgroups(self.matrix_count as u32, 1, 1);
            }

            // `[Geometry] -> Target` render pass
            {
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        /* position_id */
                        Some(wgpu::RenderPassColorAttachment {
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                            view: &self.target.position_id.get_view(),
                        }),
                        /* normal_id */
                        Some(wgpu::RenderPassColorAttachment {
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                            view: &self.target.normal_id.get_view(),
                        }),
                        /* color_opacity */
                        Some(wgpu::RenderPassColorAttachment {
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                            view: &self.target.base_color_opcaity.get_view(),
                        }),
                        /* metallic_roughness_occlusion_meta */
                        Some(wgpu::RenderPassColorAttachment {
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                            view: &self.target.metallic_roughness_occlusion_meta.get_view(),
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                        view: &self.target.depth.get_view(),
                    }),
                    ..Default::default()
                });
                let ext = self.surface.get_extent();
                render_pass.set_viewport(0.0, 0.0, ext.x as f32, ext.y as f32, 0.0, 1.0);

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

            // `Target -> Screen` render pass
            {
                // Apply lights
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store
                        },
                        resolve_target: None,
                        view: &surface_texture_view,
                    })],
                    ..Default::default()
                });

                render_pass.set_pipeline(&self.directional_light_pipeline);
                render_pass.set_bind_group(0, &self.target_bind_group, &[]);
                render_pass.set_bind_group(1, &self.directional_light_system_bind_group, &[]);
                for light in &scene.directional_lights {
                    render_pass.set_bind_group(2, &light.bind_group, &[]);
                    render_pass.draw(0..4, 0..1);
                }
            }

            let command_buffer = command_encoder.finish();

            self.kernel.queue.submit([command_buffer]);
            surface_texture.present();
        }
    }
}

// To-display primitive collection
pub struct Scene<'a> {
    primitives: Vec<&'a Primitive>,
    directional_lights: Vec<&'a DirectionalLight>,
}

impl<'a> Scene<'a> {
    /// Primitive displaying function
    /// * `primitive` - primitive to display scene-lifetime reference
    pub fn add_primitive(&mut self, primitive: &'a mut Primitive) {
        self.primitives.push(primitive);
    } // fn draw_primitive

    /// Directional light to scene adding function
    /// * `light` - light to add to scene
    pub fn add_directional_light(&mut self, light: &'a mut DirectionalLight) {
        self.directional_lights.push(light);
    } // fn add_directional_light
} // impl Scene

// file mod.rs