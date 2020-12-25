use cgmath::Point2;
use std::time::{Duration, Instant};
use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub trait Application: 'static + Sized {
    fn init(
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;

    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

    fn key_event(&mut self, key: event::VirtualKeyCode, state: event::ElementState);

    fn scroll_event(&mut self, delta: f32);

    fn mouse_moved(&mut self, pos: Point2<f32>);

    fn click_event(
        &mut self,
        button: event::MouseButton,
        state: event::ElementState,
        pos: Point2<f32>,
    );

    fn fixed_update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
}

struct Setup {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

async fn setup<App: Application>(title: &str) -> Setup {
    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);
    let window = builder.build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let optional_features = wgpu::Features::NON_FILL_POLYGON_MODE;
    let required_features = wgpu::Features::empty();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this application: {:?}",
        required_features - adapter_features
    );

    let needed_limits = wgpu::Limits::default();

    let trace_dir = std::env::var("WGPU_TRACE");
    println!("Trace Directory: {:?}", trace_dir);
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();

    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }
}

fn start<App: Application>(
    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }: Setup,
) {
    let (mut pool, _) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let mut app = App::init(&sc_desc, &device, &queue);
    let mut last_update_inst = Instant::now();
    let mut mouse_pos: Point2<f32> = Point2::new(0.0, 0.0);
    let fps = 60;

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter); // force ownership by the closure
        match event {
            event::Event::MainEventsCleared => {
                if last_update_inst.elapsed() > Duration::from_nanos(1_000_000_000 / fps) {
                    app.fixed_update(&device, &queue);
                    last_update_inst = Instant::now();
                }
                window.request_redraw();

                pool.run_until_stalled();
            }
            event::Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                app.resize(&sc_desc, &device, &queue);
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            event::Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    app.key_event(*key, *state);
                }
                WindowEvent::MouseWheel {
                    delta: event::MouseScrollDelta::LineDelta(_, delta),
                    ..
                } => {
                    app.scroll_event(*delta);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_pos = Point2::new(position.x as f32, position.y as f32);
                    app.mouse_moved(mouse_pos);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    app.click_event(*button, *state, mouse_pos);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            event::Event::RedrawRequested(_) => {
                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(_) => {
                        swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        swap_chain
                            .get_current_frame()
                            .expect("Failed to acquire next swap chain texture!")
                    }
                };

                app.render(&frame.output, &device, &queue);
            }
            _ => {}
        }
    });
}

pub fn run<App: Application>(title: &str) {
    let setup = futures::executor::block_on(setup::<App>(title));
    start::<App>(setup);
}
