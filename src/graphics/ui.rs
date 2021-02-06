use cgmath::{Point2, Vector4};
use image::GenericImageView;
use std::collections::HashMap;
use std::mem;
use wgpu::util::DeviceExt;
use crate::ui::widget_textures;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GPUSprite {
    pub pos: Vector4<f32>,
    pub uvs: Vector4<f32>,
    pub color: Vector4<f32>,
}

unsafe impl bytemuck::Pod for GPUSprite {}
unsafe impl bytemuck::Zeroable for GPUSprite {}

pub struct UiTexture {
    pub bind_group: wgpu::BindGroup,
    pub sprite_buffer: wgpu::Buffer,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct UiTextureId(usize);

#[derive(Clone, Copy)]
pub struct UiTextureRegion {
    pub texture_id: UiTextureId,
    pub pos: Point2<f32>,
    pub size: Point2<f32>,
}

impl UiTextureRegion {
    fn sub_texture(&self, texture: (f32, f32, f32, f32)) -> Self {
        Self {
            texture_id: self.texture_id,
            pos: Point2::new(texture.0, texture.1),
            size: Point2::new(texture.2, texture.3),
        }
    }
}

pub struct UiRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub camera: UiCamera,
    pub texture_arena: TextureArena,
    pub batch: UiBatch,
}

#[derive(Clone)]
pub struct UiTextures {
    pub button: UiTextureRegion,
    pub button_pressed: UiTextureRegion,
}

impl UiRenderer {
    const MAX_SPRITES: u64 = 1024;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let camera = UiCamera::new(device, swapchain);

        let vertex_bytes = super::read_file_bytes("assets/shaders/ui.vert.spv");
        let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("UI Vertex Shader"),
            source: wgpu::util::make_spirv(&vertex_bytes),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let fragment_bytes = super::read_file_bytes("assets/shaders/ui.frag.spv");
        let fragment_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("UI Fragment Shader"),
            source: wgpu::util::make_spirv(&fragment_bytes),
            flags: wgpu::ShaderFlags::VALIDATION,
        });
        let mut texture_arena = TextureArena::new(device);
        let atlas = texture_arena.load_texture(device, queue, "assets/ui/widgets.png");
        let textures = UiTextures {
            button: atlas.sub_texture(widget_textures::BUTTON),
            button_pressed: atlas.sub_texture(widget_textures::BUTTON_PRESSED),
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout, &texture_arena.bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            multisample: wgpu::MultisampleState {
                count: crate::MSAA_SAMPLE,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: None,
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<GPUSprite>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float2,
                        1 => Float2,
                        2 => Float2,
                        3 => Float2,
                        4 => Float4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swapchain.format,
                    color_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });
        let batch = UiBatch::new(textures);

        Self {
            pipeline,
            camera,
            texture_arena,
            batch,
        }
    }

    pub fn textures(&self) -> UiTextures {
        self.batch.textures.clone()
    }
}

pub struct TextureArena {
    arena: Vec<UiTexture>,
    bg_layout: wgpu::BindGroupLayout,
}

impl TextureArena {
    fn new(device: &wgpu::Device) -> Self {
        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
            label: Some("UiTextureBindGroupLayout"),
        });

        Self {
            arena: Vec::new(),
            bg_layout,
        }
    }

    pub fn get(&self, id: UiTextureId) -> &UiTexture {
        self.arena.get(id.0).unwrap()
    }

    fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &'static str,
    ) -> UiTextureRegion {
        let bytes = super::read_file_bytes(path);
        let image = image::load_from_memory(&bytes).unwrap();
        let image_rgba = image.as_rgba8().unwrap();
        let (width, height) = image.dimensions();
        let id = self.arena.len();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some(&format!("UiTexture({})", id)),
        });
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytemuck::cast_slice(&image_rgba),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: width * 4,
                rows_per_image: height,
            },
            wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
        );

        let sprite_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("UiSpriteBuffer({})", id)),
            size: UiRenderer::MAX_SPRITES * mem::size_of::<GPUSprite>() as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        println!("Loaded UiTexture({}): {}", id, path);
        self.arena.push(UiTexture {
            sprite_buffer,
            bind_group,
        });

        UiTextureRegion {
            texture_id: UiTextureId(id),
            pos: Point2::new(0.0, 0.0),
            size: Point2::new(1.0, 1.0),
        }
    }
}

pub struct UiCamera {
    pub bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
}

impl UiCamera {
    pub fn new(device: &wgpu::Device, swapchain: &wgpu::SwapChainDescriptor) -> Self {
        let matrix = UiCamera::to_matrix(swapchain.width as f32, swapchain.height as f32);
        let buffer_size = mem::size_of::<super::CameraMatrix>() as u64;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Camera Buffer"),
            contents: bytemuck::cast_slice(&[matrix]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(std::num::NonZeroU64::new(buffer_size).unwrap()),
                },
                count: None,
            }],
            label: Some("UI Camera Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &buffer,
                    offset: 0,
                    size: Some(std::num::NonZeroU64::new(buffer_size).unwrap()),
                },
            }],
            label: Some("UI Camera Bind Group"),
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, swapchain: &wgpu::SwapChainDescriptor) {
        let matrix = UiCamera::to_matrix(swapchain.width as f32, swapchain.height as f32);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[matrix]));
    }

    fn to_matrix(screen_width: f32, screen_height: f32) -> super::CameraMatrix {
        let matrix = cgmath::ortho(0.0, screen_width, 0.0, screen_height, 10.0, -10.0);

        super::CameraMatrix(super::Camera::OPENGL_TO_WGPU_MATRIX * matrix)
    }
}

pub struct UiBatch {
    pub textures: UiTextures,
    sprites: HashMap<UiTextureId, Vec<GPUSprite>>,
}

impl UiBatch {
    fn new(textures: UiTextures) -> Self {
        Self {
            textures,
            sprites: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.sprites.clear();
    }

    pub fn draw(&mut self, texture: UiTextureId, sprite: &GPUSprite) {
        if self.sprites.contains_key(&texture) {
            self.sprites.get_mut(&texture).unwrap().push(*sprite);
        } else {
            self.sprites.insert(texture, vec![*sprite]);
        }
    }

    pub fn sprites(&self) -> &HashMap<UiTextureId, Vec<GPUSprite>> {
        &self.sprites
    }
}
