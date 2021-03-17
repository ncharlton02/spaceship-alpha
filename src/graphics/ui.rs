use cgmath::{Point2, Vector4};
use image::GenericImageView;
use rusttype::{Font, Scale};
use std::collections::HashMap;
use std::mem;
use wgpu::util::DeviceExt;

// The characters that are pre-rendered by the game.
const FONT_CHARACTERS: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
    abcdefghijklmnopqrstuvwxyz\
    1234567890\
    !`?'.,;:()[]{}<>|/@\\^$-%+=#_&~*";
// const FONT_CHARACTERS: &'static str = "hello";

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GPUSprite {
    pub pos: Vector4<f32>,
    pub uvs: Vector4<f32>,
    pub color: Vector4<f32>,
}

unsafe impl bytemuck::Pod for GPUSprite {}
unsafe impl bytemuck::Zeroable for GPUSprite {}

#[derive(Clone, Copy, Debug)]
pub struct TextureRegion2D {
    pub pos: Point2<f32>,
    pub size: Point2<f32>,
}

pub struct UiRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub camera: UiCamera,
    pub batch: UiBatch,
}

impl UiRenderer {
    const MAX_SPRITES: u64 = 1024;

    pub fn new(device: &wgpu::Device, swapchain: &wgpu::SwapChainDescriptor) -> Self {
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

        let mut texture_atlas = TextureAtlas::new(device);
        let dot = texture_atlas.load_texture("assets/ui/widgets/dot.png");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout, &texture_atlas.bg_layout],
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
        let batch = UiBatch {
            atlas: texture_atlas,
            sprites: Vec::new(),
            dot,
        };

        Self {
            pipeline,
            camera,
            batch,
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
    pub atlas: TextureAtlas,
    dot: TextureRegion2D,
    sprites: Vec<GPUSprite>,
}

impl UiBatch {
    pub fn reset(&mut self) {
        self.sprites.clear();
    }

    pub fn draw(&mut self, pos: Vector4<f32>, uvs: TextureRegion2D, color: Vector4<f32>) {
        if let Some(size) = self.atlas.size {
            self.sprites.push(GPUSprite {
                pos,
                color,
                uvs: Vector4::new(
                    uvs.pos.x / size.x,
                    uvs.pos.y / size.y,
                    uvs.size.x / size.x,
                    uvs.size.y / size.y,
                ),
            });
        }
    }

    pub fn rect(&mut self, pos: Vector4<f32>, color: Vector4<f32>) {
        self.draw(pos, self.dot, color);
    }

    pub fn sprites(&self) -> &[GPUSprite] {
        &self.sprites
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FontGlyph {
    pub texture: TextureRegion2D,
    pub width: f32,
    pub height: f32,
    pub advance_width: f32,
    pub descent: f32,
}

// TODO - Remove clone requirement and do not store
// uneeded copy in the UIBatch
#[derive(Clone)]
pub struct FontMap {
    scale: Scale,
    font: Font<'static>,
    map: HashMap<char, FontGlyph>,
}

impl FontMap {
    pub fn char(&self, c: char) -> FontGlyph {
        *self
            .map
            .get(&c)
            .expect(&format!("Invalid character: {}", c))
    }

    pub fn pair_kerning(&self, last: char, current: char) -> f32 {
        self.font.pair_kerning(self.scale, last, current)
    }
}

#[derive(Clone, Copy)]
pub struct NinePatch {
    pub bottom_left: TextureRegion2D,
    pub bottom_center: TextureRegion2D,
    pub bottom_right: TextureRegion2D,
    pub middle_left: TextureRegion2D,
    pub middle_center: TextureRegion2D,
    pub middle_right: TextureRegion2D,
    pub top_left: TextureRegion2D,
    pub top_center: TextureRegion2D,
    pub top_right: TextureRegion2D,
}

pub struct TextureAtlas {
    pub size: Option<Point2<f32>>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub sprite_buffer: wgpu::Buffer,
    packer: texture_packer::TexturePacker<'static, image::DynamicImage>,
    bg_layout: wgpu::BindGroupLayout,
    texture_count: u32,
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let sprite_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TextureAtlasSprites"),
            size: UiRenderer::MAX_SPRITES * mem::size_of::<GPUSprite>() as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

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
            label: Some("UiTextureAtlasBindGroupLayout"),
        });

        let packer =
            texture_packer::TexturePacker::new_skyline(texture_packer::TexturePackerConfig {
                allow_rotation: false,
                max_width: 2048,
                max_height: 2048,
                texture_extrusion: 1,
                trim: false,
                ..Default::default()
            });

        Self {
            packer,
            sprite_buffer,
            bg_layout,
            bind_group: None,
            size: None,
            texture_count: 0,
        }
    }

    pub fn load_ninepatch(&mut self, path: &'static str) -> NinePatch {
        let bytes = super::read_file_bytes(path);
        let image = image::load_from_memory(&bytes).unwrap();
        let image_rgba = image.as_rgba8().unwrap();
        let (width, height) = image.dimensions();

        let mut x_space: Option<(u32, u32)> = None;
        let mut y_space: Option<(u32, u32)> = None;
        for x in 0..width {
            if image_rgba.get_pixel(x, 0).0 == [0, 0, 0, 255] {
                if let Some(x_space) = &mut x_space {
                    x_space.1 = x;
                } else {
                    x_space = Some((x, 0));
                }
            }
        }
        for y in 0..height {
            if image_rgba.get_pixel(0, y).0 == [0, 0, 0, 255] {
                if let Some(y_space) = &mut y_space {
                    y_space.1 = y;
                } else {
                    y_space = Some((y, 0));
                }
            }
        }

        let mut add_subtexture = |name: &'static str, pt1: Point2<u32>, pt2: Point2<u32>| {
            let sub_image = copy_subtexture(&image_rgba, pt1, pt2);

            self.add_texture(&format!("{}-{}", path, name), sub_image)
        };
        let x_space = x_space.expect("Invalid Ninepatch: No X-Axis Marker!");
        let y_space = y_space.expect("Invalid Ninepatch: No Y-Axis Marker!");
        println!("NinepatchX: {:?}", x_space);
        println!("NinepatchY: {:?}", y_space);

        //We need to go from pixel (1, 1) to (width - 1, height -1) to
        //remove the marker pixel
        NinePatch {
            bottom_left: add_subtexture(
                "BottomLeft",
                Point2::new(1, 1),
                Point2::new(x_space.0, y_space.0),
            ),
            bottom_center: add_subtexture(
                "BottomCenter",
                Point2::new(x_space.0, 1),
                Point2::new(x_space.1, y_space.0),
            ),
            bottom_right: add_subtexture(
                "BottomRight",
                Point2::new(x_space.1, 1),
                Point2::new(width - 1, y_space.0),
            ),
            middle_left: add_subtexture(
                "MiddleLeft",
                Point2::new(1, y_space.0),
                Point2::new(x_space.0, y_space.1),
            ),
            middle_center: add_subtexture(
                "MiddleCenter",
                Point2::new(x_space.0, y_space.0),
                Point2::new(x_space.1, y_space.1),
            ),
            middle_right: add_subtexture(
                "MiddleRight",
                Point2::new(x_space.1, y_space.0),
                Point2::new(width - 1, y_space.1),
            ),
            top_left: add_subtexture(
                "TopLeft",
                Point2::new(1, y_space.1),
                Point2::new(x_space.0, height - 1),
            ),
            top_center: add_subtexture(
                "TopCenter",
                Point2::new(x_space.0, y_space.1),
                Point2::new(x_space.1, height - 1),
            ),
            top_right: add_subtexture(
                "TopRight",
                Point2::new(x_space.1, y_space.1),
                Point2::new(width - 1, height - 1),
            ),
        }
    }

    pub fn load_font(&mut self, path: &'static str) -> FontMap {
        use rusttype::{point, Rect};

        let padding = 2.0;
        let bytes = super::read_file_bytes(path);
        let font = Font::try_from_vec(bytes).unwrap();
        //TODO: Currently this cannot change because
        //the size for space is hard-coded into TextLayoutRenderer
        let size = 32.0;
        let scale = Scale::uniform(size);
        let v_metrics = font.v_metrics(scale);
        let baseline = padding + v_metrics.ascent;
        let glyphs: Vec<_> = font
            .layout(FONT_CHARACTERS, scale, point(padding, baseline))
            .collect();
        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
        let glyphs_width = {
            let min_x = glyphs
                .first()
                .map(|g| g.pixel_bounding_box().unwrap().min.x)
                .unwrap();
            let max_x = glyphs
                .last()
                .map(|g| g.pixel_bounding_box().unwrap().max.x)
                .unwrap();
            (max_x - min_x) as u32
        };

        let texture_width = glyphs_width + (padding as u32 * 2);
        let texture_height = glyphs_height + (padding as u32 * 2);
        let mut image = image::DynamicImage::new_rgba8(texture_width, texture_height).to_rgba8();

        for glyph in &glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    image.put_pixel(
                        x + bounding_box.min.x as u32,
                        y + bounding_box.min.y as u32,
                        image::Rgba([255, 255, 255, (v * 255.0) as u8]),
                    )
                });
            }
        }

        let texture = self
            .add_texture(path, image::DynamicImage::ImageRgba8(image))
            .pos;

        let font_glyphs = glyphs
            .into_iter()
            .filter(|glyph| glyph.pixel_bounding_box().is_some())
            .map(|glyph| {
                let h_metrics = glyph.unpositioned().h_metrics();
                let bbox = glyph.pixel_bounding_box().unwrap();
                let bbox = Rect {
                    min: point(bbox.min.x as f32, bbox.min.y as f32),
                    max: point(bbox.max.x as f32, bbox.max.y as f32),
                };

                FontGlyph {
                    width: bbox.max.x - bbox.min.x,
                    height: bbox.max.y - bbox.min.y,
                    advance_width: h_metrics.advance_width - h_metrics.left_side_bearing,
                    descent: baseline - bbox.max.y,
                    // We need render bottom up, so we flip y min/max here
                    texture: TextureRegion2D {
                        pos: Point2::new(bbox.min.x + texture.x, bbox.max.y + texture.y),
                        size: Point2::new(bbox.max.x - bbox.min.x, bbox.min.y - bbox.max.y),
                    },
                }
            });

        let map = FONT_CHARACTERS.chars().zip(font_glyphs).collect();

        FontMap { font, scale, map }
    }

    #[allow(dead_code)]
    pub fn load_texture(&mut self, path: &str) -> TextureRegion2D {
        let bytes = super::read_file_bytes(path);
        let image = image::load_from_memory(&bytes).unwrap();

        self.add_texture(path, image)
    }

    pub fn add_texture(&mut self, name: &str, image: image::DynamicImage) -> TextureRegion2D {
        self.packer.pack_own(name.to_owned(), image).unwrap();
        let frame = self
            .packer
            .get_frame(&name)
            .expect("Unable to pack texture atlas!")
            .frame;

        TextureRegion2D {
            pos: Point2::new(frame.x as f32, frame.y as f32),
            size: Point2::new(frame.w as f32, frame.h as f32),
        }
    }

    pub fn update_gpu_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let image = texture_packer::exporter::ImageExporter::export(&self.packer).unwrap();
        let image_rgba = image.as_rgba8().unwrap();
        let (width, height) = image.dimensions();

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
            label: Some(&format!("TextureAtlas({})", self.texture_count)),
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

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        self.size = Some(Point2::new(width as f32, height as f32));
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            label: Some(&format!("TextureAtlasBindGroup({})", self.texture_count)),
        }));

        println!("Updated Texture Atlas! Index = {}", self.texture_count);
        self.texture_count += 1;
    }
}

fn copy_subtexture(
    original: &image::RgbaImage,
    p1: Point2<u32>,
    p2: Point2<u32>,
) -> image::DynamicImage {
    let width = p2.x - p1.x;
    let height = p2.y - p1.y;
    let mut new = image::DynamicImage::new_rgba8(width, height).to_rgba8();

    for x in 0..width {
        for y in 0..height {
            new.put_pixel(x, y, *original.get_pixel(x + p1.x, y + p1.y));
        }
    }

    image::DynamicImage::ImageRgba8(new)
}
