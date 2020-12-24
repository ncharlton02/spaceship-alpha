use cgmath::{Matrix4, Point3, Vector3};
use std::mem;
use wgpu::util::DeviceExt;

pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn rectangular_prism(x: f32, y: f32, z: f32, color: Point3<f32>) -> Mesh {
        let vertices = vec![
            // Bottom
            Vertex::new(0.0, y, 0.0, Point3::new(0.0, 0.0, -1.0), color),
            Vertex::new(0.0, 0.0, 0.0, Point3::new(0.0, 0.0, -1.0), color),
            Vertex::new(x, 0.0, 0.0, Point3::new(0.0, 0.0, -1.0), color),
            Vertex::new(x, y, 0.0, Point3::new(0.0, 0.0, -1.0), color),
            // Top
            Vertex::new(0.0, y, z, Point3::new(0.0, 0.0, 1.0), color),
            Vertex::new(0.0, 0.0, z, Point3::new(0.0, 0.0, 1.0), color),
            Vertex::new(x, 0.0, z, Point3::new(0.0, 0.0, 1.0), color),
            Vertex::new(x, y, z, Point3::new(0.0, 0.0, 1.0), color),
            // Left
            Vertex::new(0.0, y, z, Point3::new(0.0, -1.0, 0.0), color),
            Vertex::new(0.0, y, 0.0, Point3::new(0.0, -1.0, 0.0), color),
            Vertex::new(0.0, 0.0, 0.0, Point3::new(0.0, -1.0, 0.0), color),
            Vertex::new(0.0, 0.0, z, Point3::new(0.0, -1.0, 0.0), color),
            //Right
            Vertex::new(x, y, z, Point3::new(0.0, 1.0, 0.0), color),
            Vertex::new(x, y, 0.0, Point3::new(0.0, 1.0, 0.0), color),
            Vertex::new(x, 0.0, 0.0, Point3::new(0.0, 1.0, 0.0), color),
            Vertex::new(x, 0.0, z, Point3::new(0.0, 1.0, 0.0), color),
            //Front
            Vertex::new(x, y, z, Point3::new(1.0, 0.0, 0.0), color),
            Vertex::new(x, y, 0.0, Point3::new(1.0, 0.0, 0.0), color),
            Vertex::new(0.0, y, 0.0, Point3::new(1.0, 0.0, 0.0), color),
            Vertex::new(0.0, y, z, Point3::new(1.0, 0.0, 0.0), color),
            //Back
            Vertex::new(x, 0.0, z, Point3::new(-1.0, 0.0, 0.0), color),
            Vertex::new(x, 0.0, 0.0, Point3::new(-1.0, 0.0, 0.0), color),
            Vertex::new(0.0, 0.0, 0.0, Point3::new(-1.0, 0.0, 0.0), color),
            Vertex::new(0.0, 0.0, z, Point3::new(-1.0, 0.0, 0.0), color),
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

pub struct MeshId(usize);

pub struct MeshRegistry {
    meshes: Vec<GPUMesh>,
}

impl MeshRegistry {
    pub fn add(&mut self, device: &wgpu::Device, mesh: &Mesh) -> MeshId {
        let id = self.meshes.len();
        let gpu_mesh = GPUMesh::create(device, mesh, id);
        self.meshes.push(gpu_mesh);

        println!("[Registered Mesh] {}={}", &mesh.name, id);

        MeshId(id)
    }

    /// Note: queue.submit() needs to be called after this for the changes to take affect
    pub fn write_models(&mut self, queue: &wgpu::Queue, mesh: MeshId, models: &[Matrix4<f32>]){
        //We need to place the matrices in a struct that we can mark as Pod / Zeroable
        let models: Vec<ModelMatrix> = models.iter().map(|matrix| ModelMatrix(*matrix)).collect();
        let mesh = self.meshes.get_mut(mesh.0).unwrap_or_else(|| panic!("Invalid mesh ID: {}", mesh.0));
        mesh.instances = models.len() as u32;
    
        queue.write_buffer(&mesh.models_buffer, 0, bytemuck::cast_slice(&models));
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
    mesh_registry: MeshRegistry,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, swapchain_format: &wgpu::TextureFormat) -> Renderer {
        let mesh_registry = MeshRegistry { meshes: Vec::new() };
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
                format: *swapchain_format,
                color_blend: wgpu::BlendDescriptor::default(),
                alpha_blend: wgpu::BlendDescriptor::default(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
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
            mesh_registry,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &wgpu::SwapChainTexture,
        camera: &Camera,
    ) {
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
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.camera_bg, &[]);

        for mesh in &self.mesh_registry.meshes {
            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, mesh.models_buffer.slice(..));
            rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..mesh.index_count, 0, 0..mesh.instances);
        }

        std::mem::drop(rpass);
        queue.submit(Some(encoder.finish()));
    }

    pub fn mesh_registry(&mut self) -> &mut MeshRegistry {
        &mut self.mesh_registry
    }
}

#[derive(Clone, Copy)]
struct CameraMatrix(Matrix4<f32>);

unsafe impl bytemuck::Pod for CameraMatrix {}
unsafe impl bytemuck::Zeroable for CameraMatrix {}

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
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

    fn build_view_projection_matrix(&self) -> CameraMatrix {
        let view = Matrix4::look_at(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.near, self.far);

        CameraMatrix(Self::OPENGL_TO_WGPU_MATRIX * proj * view)
    }
}
