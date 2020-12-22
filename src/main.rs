use cgmath::Point2;
use winit::event;
use renderer::Renderer;

mod app;
mod renderer;

struct AppState {
    renderer: Renderer,
}

impl app::Application for AppState {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, _: &wgpu::Queue) -> Self {
        let renderer = Renderer::new(device, &swapchain.format);
        AppState {
            renderer
        }
    }

    fn resize(&mut self, _: &wgpu::SwapChainDescriptor, _: &wgpu::Device, _: &wgpu::Queue) {}

    fn key_event(&mut self, _: event::VirtualKeyCode, _: event::ElementState) {}

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, _: Point2<f32>) {}

    fn click_event(&mut self, _: event::MouseButton, _: event::ElementState, _: Point2<f32>) {}

    fn render(&mut self, texture: &wgpu::SwapChainTexture, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.renderer.render(device, queue, texture)
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}
