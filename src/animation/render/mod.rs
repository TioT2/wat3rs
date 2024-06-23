use std::{borrow::Cow, cell::Cell, sync::{Arc, Weak}};

mod event;
pub mod camera;

use camera::Camera;
use event::{EventSender, EventReciever};

use crate::util::{Mat4x4f, Vec2f, Vec3f};

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub position: Vec3f,
    pub tex_coord: Vec2f,
    pub normal: Vec3f,
}

unsafe impl bytemuck::NoUninit for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Copy, Clone)]
pub struct GeometryAccessor {
    pub offset: usize,
    pub size: usize,
    pub count: u32,
}

struct PrimitiveImpl {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_accessor: GeometryAccessor,
    index_accessor: Option<GeometryAccessor>,
    transform: Mat4x4f,
    id: Arc<Cell<u32>>,
}

struct GeometryImpl {
    geometry_buffer: wgpu::Buffer,
    primitives: Vec<PrimitiveImpl>,
    id: Arc<Cell<u32>>,
}

/// Geometry implementaiton structure
pub struct Geometry {
    sender: EventSender<GeometryEvent>,
    geometry_id: Arc<Cell<u32>>,
}

impl Geometry {
    /// Primitive create function
    /// * `descriptor` - primitive descriptor
    pub fn create_primitive(&self, descriptor: &PrimitiveDescriptor) -> Primitive {
        let id_dst = Arc::new(Cell::new(u32::MAX));

        self.sender.add_event(GeometryEvent {
            id: self.geometry_id.get(),
            ty: GeometryEventType::CreatePrimitive {
                id_dst: id_dst.clone(),
                descriptor: Box::new(*descriptor),
            }
        });

        Primitive {
            geometry_id: self.geometry_id.clone(),
            primitive_id: id_dst,
            sender: self.sender.clone(),
        }
    } // fn create_primitive
}

/// Primitve representation structure
pub struct Primitive {
    sender: EventSender<GeometryEvent>,
    primitive_id: Arc<Cell<u32>>,
    geometry_id: Arc<Cell<u32>>,
}

impl Primitive {
    pub fn set_transform(&self, transform: Mat4x4f) {
        self.sender.add_event(GeometryEvent {
            id: self.geometry_id.get(),
            ty: GeometryEventType::SetTransform { transform, primitive_id: self.primitive_id.clone() }
        });
    } // fn set_transform

    pub fn set_base_color(&self, base_color: Vec3f) {
        self.sender.add_event(GeometryEvent {
            id: self.geometry_id.get(),
            ty: GeometryEventType::SetBaseColor { base_color, primitive_id: self.primitive_id.clone() },
        });
    }
} // impl Primitive

impl Clone for Primitive {
    fn clone(&self) -> Self {
        let id = Arc::new(Cell::new(0u32));

        self.sender.add_event(GeometryEvent {
            id: self.geometry_id.get(),
            ty: GeometryEventType::ClonePrimitive { src_id: self.primitive_id.clone(), dst_id: id.clone() },
        });

        Self {
            sender: self.sender.clone(),
            geometry_id: self.geometry_id.clone(),
            primitive_id: id,
        }
    }
}

enum GeometryEventType {
    SetTransform {
        transform: Mat4x4f,
        primitive_id: Arc<Cell<u32>>,
    },
    SetBaseColor {
        base_color: Vec3f,
        primitive_id: Arc<Cell<u32>>,
    },
    CreatePrimitive {
        id_dst: Arc<Cell<u32>>,
        descriptor: Box<PrimitiveDescriptor>,
    },
    ClonePrimitive {
        src_id: Arc<Cell<u32>>,
        dst_id: Arc<Cell<u32>>,
    },
}

// Primitive IDs aren't tended to be changed before ALL events handled.
struct GeometryEvent {
    pub id: u32,
    pub ty: GeometryEventType,
}

/// Render system representation structure
pub struct Render {
    camera: Camera,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    reciever: EventReciever<GeometryEvent>,
    depth_buffer: wgpu::Texture,

    geometries: Vec<GeometryImpl>,
    geometry_pipeline: wgpu::RenderPipeline,
    geometry_bind_group_layout: wgpu::BindGroupLayout,
} // struct Render

/// Geometry subset descriptor
#[derive(Copy, Clone)]
pub struct PrimitiveDescriptor {
    pub vertex_accessor: GeometryAccessor,
    pub index_accessor: Option<GeometryAccessor>,
    pub color: Vec3f,
} // struct PrimitiveDescriptor

/// Geometry descriptor
pub struct GeometryDescriptor<'t> {
    pub geometry_buffer: &'t [u8],

    pub primitives: &'t [PrimitiveDescriptor],
} // struct GeometryDescriptor

#[repr(C)]
#[derive(Copy, Clone)]
struct PrimitiveBufferMatrices {
    pub world_view_projection: Mat4x4f,
    pub world: Mat4x4f,
    pub world_inverse: Mat4x4f,
}

impl Default for PrimitiveBufferMatrices {
    fn default() -> Self {
        Self {
            world: Mat4x4f::identity(),
            world_inverse: Mat4x4f::identity(),
            world_view_projection: Mat4x4f::identity(),
        }
    }
}

unsafe impl bytemuck::NoUninit for PrimitiveBufferMatrices {}

/// At least now world_view_projection and world_inverse matrices will be calculated at CPU and written into individual buffers every frame
#[repr(C)]
#[derive(Copy, Clone, Default)]
struct PrimitiveBufferData {
    pub matrices: PrimitiveBufferMatrices,
    pub base_color: Vec3f,
    pub primitive_id: u32,
}

unsafe impl bytemuck::NoUninit for PrimitiveBufferData {}
unsafe impl bytemuck::Zeroable for PrimitiveBufferData {}

impl Render {
    pub fn new(window: Arc<winit::window::Window>) -> Option<Render> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).ok()?;

        let adapter = futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;

        let window_extent = window.inner_size();

        let (device, queue) = futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            ..Default::default()
        }, None)).ok()?;

        let surface_config = surface.get_default_config(&adapter, window_extent.width, window_extent.height)?;

        surface.configure(&device, &surface_config);

        let primitive_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some((std::mem::size_of::<PrimitiveBufferData>() as u64).try_into().unwrap()),
                }
            }]
        });

        let render_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/render.wgsl")))
        });

        let primitive_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&primitive_bind_group_layout],
        });

        let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: window_extent.width,
                height: window_extent.height,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });

        let primitive_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main render pipeline"),
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState::default(),
                depth_compare: wgpu::CompareFunction::Less,
                depth_write_enabled: true,
                format: wgpu::TextureFormat::Depth32Float,
                stencil: wgpu::StencilState::default()
            }),
            fragment: Some(wgpu::FragmentState {
                module: &render_shader_module,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    blend: None,
                    format: surface_config.format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            vertex: wgpu::VertexState {
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: bytemuck::offset_of!(Vertex, position) as u64,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: bytemuck::offset_of!(Vertex, tex_coord) as u64,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: bytemuck::offset_of!(Vertex, normal) as u64,
                                shader_location: 2,
                            },
                        ],
                    },
                ],
                module: &render_shader_module,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default()
            },
            layout: Some(&primitive_pipeline_layout),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
        });

        Some(Render {
            camera: Camera::default(),
            surface,
            geometry_pipeline: primitive_pipeline,
            device,
            geometry_bind_group_layout: primitive_bind_group_layout,
            geometries: Vec::new(),
            queue,
            depth_buffer,
            reciever: EventReciever::new(),
            // surface_config,
        })
    } // fn new

    /// Rendering function
    pub fn render(&mut self) {
        // Update matrices, if required.
        while let Some(event) = self.reciever.poll_event() {
            match event.ty {
                GeometryEventType::SetTransform { transform, primitive_id } => {
                    // Acquire geometry and primitive
                    let geometry = self.geometries.get_mut(event.id as usize).expect(format!("Unknown geometry id: {}", event.id).as_str());
                    let primitive = geometry.primitives.get_mut(primitive_id.get() as usize).expect(format!("Unknown {} geometry primitive id: {}", event.id, primitive_id.get()).as_str());

                    // Rewrite world and world_inverse matrices
                    self.queue.write_buffer(&primitive.uniform_buffer, bytemuck::offset_of!(PrimitiveBufferMatrices, world) as u64, bytemuck::bytes_of(&transform));
                    self.queue.write_buffer(&primitive.uniform_buffer, bytemuck::offset_of!(PrimitiveBufferMatrices, world_inverse) as u64, bytemuck::bytes_of(&transform.inversed()));

                    // Set primitive transform
                    primitive.transform = transform;
                }
                GeometryEventType::SetBaseColor { base_color, primitive_id } => {
                    // Acquire geometry and primitive
                    let geometry = self.geometries.get_mut(event.id as usize).expect(format!("Unknown geometry id: {}", event.id).as_str());
                    let primitive = geometry.primitives.get_mut(primitive_id.get() as usize).expect(format!("Unknown {} geometry primitive id: {}", event.id, primitive_id.get()).as_str());

                    self.queue.write_buffer(&primitive.uniform_buffer, bytemuck::offset_of!(PrimitiveBufferData, base_color) as u64, bytemuck::bytes_of(&base_color));
                }
                // Primitive adding function
                GeometryEventType::CreatePrimitive { id_dst, descriptor } => {
                    let primitive = self.create_primitive_impl(&descriptor, id_dst);
                    let geometry = self.geometries.get_mut(event.id as usize).expect(format!("Unknown geometry id: {}", event.id).as_str());

                    primitive.id.set(geometry.primitives.len() as u32);
                    geometry.primitives.push(primitive);
                }
                GeometryEventType::ClonePrimitive { src_id, dst_id } => {
                    let mut dst_primitive = self.create_empty_primitive_impl(dst_id);
                    let geometry = self.geometries.get_mut(event.id as usize).expect(format!("Unknown geometry id: {}", event.id).as_str());
                    let id = geometry.primitives.len() as u32;
                    let src_primitive = geometry.primitives.get_mut(src_id.get() as usize).expect(format!("Unknown {} geometry primitive id: {}", event.id, src_id.get()).as_str());

                    // Write dst primitive UBO
                    let mut cmd_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                    cmd_encoder.copy_buffer_to_buffer(&src_primitive.uniform_buffer, 0, &dst_primitive.uniform_buffer, 0, src_primitive.uniform_buffer.size());
                    self.queue.submit([cmd_encoder.finish()]);
                    self.queue.write_buffer(&dst_primitive.uniform_buffer, bytemuck::offset_of!(PrimitiveBufferData, primitive_id) as u64, bytemuck::bytes_of(&id));

                    dst_primitive.id.set(id);
                    dst_primitive.vertex_accessor = src_primitive.vertex_accessor.clone();
                    dst_primitive.index_accessor = src_primitive.index_accessor.clone();

                    geometry.primitives.push(dst_primitive);
                }
            }
        }

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return
        };
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Recalculate WVP Matrices
        let vp = self.camera.get_matrices().view_projection;
        for geometry in &self.geometries {
            for primitive in &geometry.primitives {
                self.queue.write_buffer(&primitive.uniform_buffer, bytemuck::offset_of!(PrimitiveBufferMatrices, world_view_projection) as u64, bytemuck::bytes_of(&(primitive.transform * vp)));
            }
        }

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let depth_view = self.depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());
            let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    resolve_target: None,
                    view: &frame_view,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(std::f32::INFINITY),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                    view: &depth_view,
                }),
                label: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.geometry_pipeline);

            for geom in &self.geometries {
                for prim in &geom.primitives {
                    pass.set_bind_group(0, &prim.bind_group, &[]);
                    pass.set_vertex_buffer(0, geom.geometry_buffer.slice(
                        prim.vertex_accessor.offset as u64..(prim.vertex_accessor.offset + prim.vertex_accessor.size) as u64
                    ));

                    if let Some(idx_accessor) = prim.index_accessor.as_ref() {
                        pass.set_index_buffer(geom.geometry_buffer.slice(idx_accessor.offset as u64..(idx_accessor.offset + idx_accessor.size) as u64), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..idx_accessor.count, 0, 0..1);
                    } else {
                        pass.draw(0..prim.vertex_accessor.count, 0..1);
                    }
                }

            }
        }

        self.queue.submit([command_encoder.finish()]);
        frame.present();
    } // fn response

    fn create_empty_primitive_impl(&self, id: Arc<Cell<u32>>) -> PrimitiveImpl {
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<PrimitiveBufferData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                })
            }],
            label: None,
            layout: &self.geometry_bind_group_layout,
        });

        PrimitiveImpl {
            bind_group,
            transform: Mat4x4f::identity(),
            index_accessor: Some(GeometryAccessor { offset: 0, size: 0, count: 0 }),
            vertex_accessor: GeometryAccessor { offset: 0, size: 0, count: 0 },
            uniform_buffer,
            id,
        }
    }

    fn create_primitive_impl(&mut self, desc: &PrimitiveDescriptor, id: Arc<Cell<u32>>) -> PrimitiveImpl {
        let data = PrimitiveBufferData {
            base_color: desc.color,
            primitive_id: id.get(),
            matrices: PrimitiveBufferMatrices::default(),
        };

        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<PrimitiveBufferData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        // Initialize uniform buffer
        self.queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&data));

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                })
            }],
            label: None,
            layout: &self.geometry_bind_group_layout,
        });

        PrimitiveImpl {
            bind_group,
            transform: Mat4x4f::identity(),
            index_accessor: desc.index_accessor.clone(),
            vertex_accessor: desc.vertex_accessor.clone(),
            uniform_buffer,
            id,
        }
    } // fn create_primitive

    fn create_geometry_impl(&mut self, desc: &GeometryDescriptor, id: Arc<Cell<u32>>) -> GeometryImpl {
        let geometry_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: desc.geometry_buffer.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        self.queue.write_buffer(&geometry_buffer, 0, desc.geometry_buffer);

        let primitives = desc
            .primitives
            .iter()
            .enumerate()
            .map(|(id, desc)| self.create_primitive_impl(desc, Arc::new(Cell::new(id as u32))))
            .collect();

        GeometryImpl { geometry_buffer, id, primitives }
    }

    fn create_geometry(&mut self, desc: &GeometryDescriptor) -> Geometry {
        let geometry = self.create_geometry_impl(desc, Arc::new(Cell::new(self.geometries.len() as u32)));
        let handle = Geometry {
            geometry_id: geometry.id.clone(),
            sender: self.reciever.create_sender(),
        };
        self.geometries.push(geometry);
        handle
    } // fn create_primitive


    pub fn get_state(&mut self) -> State {
        State {
            render: self,
        }
    }
} // impl Render

/// Renderer state representation structure
pub struct State<'t> {
    render: &'t mut Render,
} // struct State

impl<'t> State<'t> {
    /// Geometry create function
    /// * `descriptor` - geometry descriptor
    /// * Returns created geometry handle
    pub fn create_geometry(&mut self, descriptor: &GeometryDescriptor) -> Geometry {
        self.render.create_geometry(descriptor)
    } // fn create_geometry

    /// Camera mutable reference getting function
    /// * Retunrs actual render camera reference
    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.render.camera
    } // fn get_camera_mut
} // impl State

// file mod.rs
