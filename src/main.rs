use cgmath::{prelude::*, Point2, Point3, Vector3};
use graphics::{Camera, Mesh, MeshId, Model, ModelId, ModelUpdates, Renderer};
use winit::event;

pub const WIREFRAME_MODE: bool = true;

mod app;
mod graphics;

struct AppState {
    renderer: Renderer,
    camera: Camera,
}

impl app::Application for AppState {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, _: &wgpu::Queue) -> Self {
        let mut renderer = Renderer::new(device, &swapchain);
        let camera = Camera {
            eye: (3.0, 3.0, 3.0).into(),
            target: Point3::origin(),
            up: Vector3::unit_z(),
            aspect: swapchain.width as f32 / swapchain.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        let mesh = Mesh::rectangular_prism(0.5, 0.5, 2.0, Point3::new(0.8, 0.8, 0.8));
        let mesh_id = renderer.mesh_manager().add(device, &mesh);
        renderer
            .mesh_manager()
            .new_model(mesh_id, &Model::new((0.0, 0.0, 0.0).into(), 0.0));
        let mesh2 = Mesh::rectangular_prism(0.5, 0.5, 1.0, Point3::new(1.0, 0.4, 0.4));
        let mesh_id2 = renderer.mesh_manager().add(device, &mesh2);
        renderer
            .mesh_manager()
            .new_model(mesh_id2, &Model::new((1.5, 0.0, 0.0).into(), 0.0));

        AppState { renderer, camera }
    }

    fn resize(
        &mut self,
        swapchain: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        _: &wgpu::Queue,
    ) {
        self.camera.resize(swapchain);
        self.renderer.resize(device, swapchain);
    }

    fn key_event(&mut self, _: event::VirtualKeyCode, _: event::ElementState) {}

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, _: Point2<f32>) {}

    fn click_event(&mut self, _: event::MouseButton, _: event::ElementState, _: Point2<f32>) {}

    fn fixed_update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        use cgmath::Quaternion;

        let rotation = Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Rad(0.04));
        self.camera.eye = rotation.rotate_point(self.camera.eye);
    }

    fn render(
        &mut self,
        texture: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.renderer.render(device, queue, texture, &self.camera)
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}
