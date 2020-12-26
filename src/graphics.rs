use cgmath::{prelude::*, Matrix4, Point3, Quaternion, Vector3};
use generational_arena::Arena;
use std::{borrow::BorrowMut, mem, sync::Mutex};
use wgpu::util::DeviceExt;

pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    /// Creates a rectangular prism centered at (0, 0, 0)
    pub fn rectangular_prism(mut x: f32, mut y: f32, mut z: f32, color: Point3<f32>) -> Mesh {
        x /= 2.0;
        y /= 2.0;
        z /= 2.0;

        let vertices = vec![
            // Bottom
            Vertex::new(-x, y, -z, Point3::new(0.0, 0.0, -1.0), color),
            Vertex::new(-x, -y, -z, Point3::new(0.0, 0.0, -1.0), color),
            Vertex::new(x, -y, -z, Point3::new(0.0, 0.0, -1.0), color),
            Vertex::new(x, y, -z, Point3::new(0.0, 0.0, -1.0), color),
            // Top
            Vertex::new(-x, y, z, Point3::new(0.0, 0.0, 1.0), color),
            Vertex::new(-x, -y, z, Point3::new(0.0, 0.0, 1.0), color),
            Vertex::new(x, -y, z, Point3::new(0.0, 0.0, 1.0), color),
            Vertex::new(x, y, z, Point3::new(0.0, 0.0, 1.0), color),
            // Left
            Vertex::new(-x, y, z, Point3::new(-1.0, 0.0, 0.0), color),
            Vertex::new(-x, y, -z, Point3::new(-1.0, 0.0, 0.0), color),
            Vertex::new(-x, -y, -z, Point3::new(-1.0, 0.0, 0.0), color),
            Vertex::new(-x, -y, z, Point3::new(-1.0, 0.0, 0.0), color),
            //Right
            Vertex::new(x, y, z, Point3::new(1.0, 0.0, 0.0), color),
            Vertex::new(x, y, -z, Point3::new(1.0, 0.0, 0.0), color),
            Vertex::new(x, -y, -z, Point3::new(1.0, 0.0, 0.0), color),
            Vertex::new(x, -y, z, Point3::new(1.0, 0.0, 0.0), color),
            //Front
            Vertex::new(x, y, z, Point3::new(0.0, 1.0, 0.0), color),
            Vertex::new(x, y, -z, Point3::new(0.0, 1.0, 0.0), color),
            Vertex::new(-x, y, -z, Point3::new(0.0, 1.0, 0.0), color),
            Vertex::new(-x, y, z, Point3::new(0.0, 1.0, 0.0), color),
            //Back
            Vertex::new(x, -y, z, Point3::new(0.0, -1.0, 0.0), color),
            Vertex::new(x, -y, -z, Point3::new(0.0, -1.0, 0.0), color),
            Vertex::new(-x, -y, -z, Point3::new(0.0, -1.0, 0.0), color),
            Vertex::new(-x, -y, z, Point3::new(0.0, -1.0, 0.0), color),
        ];

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2, 0, 2, 3, //Bottom
            4, 5, 6, 4, 6, 7, //Top
            8, 9, 10, 8, 10, 11, //Left
            14, 13, 12, 15, 14, 12, //Right
            16, 17, 18, 16, 18, 19, //Front
            22, 21, 20, 23, 22, 20 //Back
        ];

        Mesh {
            name: format!("RectangularPrism(x={}, y={}, z={})", x, y, z),
            indices,
            vertices,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Point3<f32>,
    pub normal: Point3<f32>,
    pub color: Point3<f32>,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, normal: Point3<f32>, color: Point3<f32>) -> Vertex {
        Self {
            pos: (x, y, z).into(),
            normal,
            color,
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Clone, Copy)]
struct ModelMatrix(Matrix4<f32>);

unsafe impl bytemuck::Pod for ModelMatrix {}
unsafe impl bytemuck::Zeroable for ModelMatrix {}

#[derive(Clone, Copy)]
pub struct MeshId(usize);

pub type ModelId = generational_arena::Index;

pub struct Model {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Model {
    pub fn new(position: Vector3<f32>, angle_z: f32) -> Self {
        Self {
            position,
            rotation: Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Rad(angle_z)),
        }
    }
}

impl Model {
    fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::from(self.rotation) * Matrix4::from_translation(self.position)
    }
}

struct ModelUpdate {
    mesh: MeshId,
    model_id: ModelId,
    matrix: Matrix4<f32>,
}

/// Used to publish model updates
#[derive(Default)]
pub struct ModelUpdates {
    updates: Mutex<Vec<ModelUpdate>>,
}

impl ModelUpdates {
    pub fn update(&self, mesh: MeshId, model_id: ModelId, model: &Model) {
        let mut updates = self.updates.lock().expect("Lock poisoned!");
        updates.borrow_mut().push(ModelUpdate {
            mesh,
            model_id,
            matrix: model.as_matrix(),
        });
    }
}

pub struct MeshManager {
    meshes: Vec<GPUMesh>,
    models: Vec<Arena<Matrix4<f32>>>,
}

impl MeshManager {
    pub fn add(&mut self, device: &wgpu::Device, mesh: &Mesh) -> MeshId {
        let id = self.meshes.len();
        let gpu_mesh = GPUMesh::create(device, mesh, id);
        self.meshes.push(gpu_mesh);
        self.models.push(Arena::new());

        println!("[Registered Mesh] {}={}", &mesh.name, id);

        MeshId(id)
    }

    pub fn new_model(&mut self, mesh: MeshId, model: &Model) -> ModelId {
        let arena = self
            .models
            .get_mut(mesh.0)
            .unwrap_or_else(|| panic!("Invalid mesh ID: {}", mesh.0));
        arena.insert(model.as_matrix())
    }

    /// Updates the mesh manager with these updates. Will be pushed to the GPU during the next render
    pub fn update_models(&mut self, updates: ModelUpdates) {
        let updates = updates.updates.into_inner().expect("Lock was poisoned");

        for update in updates {
            let arena = self
                .models
                .get_mut(update.mesh.0)
                .unwrap_or_else(|| panic!("Invalid mesh ID: {}", update.mesh.0));
            (*arena.get_mut(update.model_id).unwrap()) = update.matrix;
        }
    }

    fn update_gpu_meshes(&mut self, queue: &wgpu::Queue) {
        for (index, mesh) in &mut self.meshes.iter_mut().enumerate() {
            let models = self
                .models
                .get(index)
                .unwrap_or_else(|| panic!("Invalid mesh ID: {}", index));
            //We need to place the matrices in a struct that we can mark as Pod / Zeroable
            let models: Vec<ModelMatrix> = models
                .iter()
                .map(|arena_entry| ModelMatrix(*arena_entry.1))
                .collect();
            mesh.instances = models.len() as u32;
            queue.write_buffer(&mesh.models_buffer, 0, bytemuck::cast_slice(&models));
        }
    }
}

struct GPUMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    models_buffer: wgpu::Buffer,
    instances: u32,
}

impl GPUMesh {
    const MODEL_COUNT: u64 = 32;

    fn create(device: &wgpu::Device, mesh: &Mesh, id: usize) -> GPUMesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("VertexBuffer(Mesh={})", id)),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("IndexBuffer(Mesh={})", id)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsage::INDEX,
        });
        let models_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("ModelBuffer(Mesh={})", id)),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
            size: mem::size_of::<ModelMatrix>() as u64 * GPUMesh::MODEL_COUNT,
        });

        GPUMesh {
            vertex_buffer,
            index_buffer,
            models_buffer,
            index_count: mesh.indices.len() as u32,
            instances: 0,
        }
    }
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    camera_bg: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    mesh_manager: MeshManager,
    depth_texture: GPUTexture,
}

impl Renderer {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(device: &wgpu::Device, swapchain: &wgpu::SwapChainDescriptor) -> Renderer {
        let mesh_manager = MeshManager {
            meshes: Vec::new(),
            models: Vec::new(),
        };
        let camera_buffer_size = 16 * mem::size_of::<f32>() as u64;
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: camera_buffer_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(std::num::NonZeroU64::new(camera_buffer_size).unwrap()),
                },
                count: None,
            }],
            label: Some("Camera Bind Group Layout"),
        });

        let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: Some(std::num::NonZeroU64::new(camera_buffer_size).unwrap()),
                },
            }],
            label: Some("Camera Bind Group"),
        });

        let vertex_shader =
            device.create_shader_module(&wgpu::include_spirv!("shaders/basic.vert.spv"));
        let frag_shader =
            device.create_shader_module(&wgpu::include_spirv!("shaders/basic.frag.spv"));
        let depth_texture = create_depth_texture(device, swapchain);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_shader,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &frag_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: if crate::WIREFRAME_MODE {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: swapchain.format,
                color_blend: wgpu::BlendDescriptor::default(),
                alpha_blend: wgpu::BlendDescriptor::default(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: Renderer::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: Some(wgpu::IndexFormat::Uint16),
                vertex_buffers: &[
                wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float3],
                },
                wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<ModelMatrix>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![3 => Float4, 4 => Float4, 5 => Float4, 6 => Float4],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Renderer {
            pipeline,
            camera_bg,
            camera_buffer,
            mesh_manager,
            depth_texture,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &wgpu::SwapChainTexture,
        camera: &Camera,
    ) {
        self.mesh_manager.update_gpu_meshes(queue);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera.build_view_projection_matrix()]),
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.001,
                        g: 0.001,
                        b: 0.001,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.camera_bg, &[]);

        for mesh in &self.mesh_manager.meshes {
            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, mesh.models_buffer.slice(..));
            rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..mesh.index_count, 0, 0..mesh.instances);
        }

        std::mem::drop(rpass);
        queue.submit(Some(encoder.finish()));
    }

    pub fn resize(&mut self, device: &wgpu::Device, swapchain: &wgpu::SwapChainDescriptor) {
        self.depth_texture = create_depth_texture(device, swapchain);
    }

    pub fn mesh_manager(&mut self) -> &mut MeshManager {
        &mut self.mesh_manager
    }
}

struct GPUTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

fn create_depth_texture(
    device: &wgpu::Device,
    swapchain: &wgpu::SwapChainDescriptor,
) -> GPUTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: swapchain.width,
            height: swapchain.height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Renderer::DEPTH_FORMAT,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    GPUTexture {
        texture,
        view,
        sampler,
    }
}

#[derive(Clone, Copy)]
struct CameraMatrix(Matrix4<f32>);

unsafe impl bytemuck::Pod for CameraMatrix {}
unsafe impl bytemuck::Zeroable for CameraMatrix {}

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    #[rustfmt::skip]
    const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    pub fn resize(&mut self, swapchain: &wgpu::SwapChainDescriptor) {
        self.aspect = swapchain.width as f32 / swapchain.height as f32;
    }

    fn build_view_projection_matrix(&self) -> CameraMatrix {
        let view = Matrix4::look_at_dir(
            self.position,
            Vector3::new(self.yaw.cos(), self.yaw.sin(), self.pitch.sin()).normalize(),
            Vector3::unit_z(),
        );
        let proj = cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.near, self.far);

        CameraMatrix(Self::OPENGL_TO_WGPU_MATRIX * proj * view)
    }
}
