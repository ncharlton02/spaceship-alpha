use crate::entity::Line;
use std::mem;

pub struct LineRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
}

impl LineRenderer {
    const MAX_LINES: u64 = 32;

    pub fn new(
        device: &wgpu::Device,
        camera_bgl: &wgpu::BindGroupLayout,
        swapchain: &wgpu::SwapChainDescriptor,
    ) -> LineRenderer {
        let vertex_buffer_size = LineRenderer::MAX_LINES * mem::size_of::<Line>() as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Buffer"),
            size: vertex_buffer_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_bytes = super::load_shader("assets/shaders/line.vert.spv");
        let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Vertex"),
            source: wgpu::util::make_spirv(&vertex_bytes),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let frag_bytes = super::load_shader("assets/shaders/line.frag.spv");
        let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Fragment"),
            source: wgpu::util::make_spirv(&frag_bytes),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Pipeline Layout"),
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            multisample: wgpu::MultisampleState {
                count: crate::MSAA_SAMPLE,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::Renderer::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false,
            }),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Line>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float3],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swapchain.format,
                    color_blend: wgpu::BlendState::default(),
                    alpha_blend: wgpu::BlendState::default(),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        Self {
            vertex_buffer,
            pipeline,
        }
    }
}
