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

        let vertex_shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/line.vert.spv"));
        let frag_shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/line.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Pipeline Layout"),
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
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::LineList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: swapchain.format,
                color_blend: wgpu::BlendDescriptor::default(),
                alpha_blend: wgpu::BlendDescriptor::default(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: super::Renderer::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: None,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<Line>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float3],
                }],
            },
            sample_count: crate::MSAA_SAMPLE,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            vertex_buffer,
            pipeline,
        }
    }
}
