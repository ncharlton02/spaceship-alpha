use cgmath::{prelude::*, Matrix4, Point2, Point3, Vector3};
use graphics::{Camera, Mesh, MeshId, Renderer, Vertex};
use winit::event;

pub const WIREFRAME_MODE: bool = false;

mod app;
mod graphics;

struct AppState {
    renderer: Renderer,
    camera: Camera,
}

impl app::Application for AppState {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mesh = Mesh::rectangular_prism(0.5, 0.5, 2.0, Point3::new(0.8, 0.8, 0.8));
        let mut renderer = Renderer::new(device, &swapchain.format);
        let mesh_id = renderer.mesh_registry().add(device, &mesh);
        let camera = Camera {
            eye: (3.0, 3.0, 3.0).into(),
            target: Point3::origin(),
            up: Vector3::unit_z(),
            aspect: swapchain.width as f32 / swapchain.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        let model1 = Matrix4::identity();
        let model2 = Matrix4::from_translation((1.0, 0.0, 0.0).into());

        renderer.mesh_registry().write_models(queue, mesh_id, &vec![model1, model2]);

        AppState { renderer, camera }
    }

    fn resize(&mut self, swapchain: &wgpu::SwapChainDescriptor, _: &wgpu::Device, _: &wgpu::Queue) {
        self.camera.aspect = swapchain.width as f32 / swapchain.height as f32;
    }

    fn key_event(&mut self, _: event::VirtualKeyCode, _: event::ElementState) {}

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, _: Point2<f32>) {}

    fn click_event(&mut self, _: event::MouseButton, _: event::ElementState, _: Point2<f32>) {}

    fn render(
        &mut self,
        texture: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let rotation = cgmath::Quaternion::from_angle_z(cgmath::Rad(0.02));

        self.camera.eye = rotation.rotate_point(self.camera.eye);
        self.renderer.render(device, queue, texture, &self.camera)
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}
