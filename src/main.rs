use cgmath::{prelude::*, Point2, Point3, Vector3};
use graphics::{Camera, Mesh, MeshId, Model, ModelId, ModelUpdates, Renderer};
use winit::event;

pub const WIREFRAME_MODE: bool = false;

mod app;
mod graphics;

struct AppState {
    renderer: Renderer,
    camera: Camera,
    angle: f32,
    mesh_id: MeshId,
    model_id: ModelId,
}

impl app::Application for AppState {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, _: &wgpu::Queue) -> Self {
        let mut renderer = Renderer::new(device, &swapchain.format);
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
        let model_id = renderer
            .mesh_manager()
            .new_model(mesh_id, &Model::new((0.0, 0.0, 0.0).into(), 0.0));

        AppState {
            renderer,
            camera,
            model_id,
            mesh_id,
            angle: 0.0,
        }
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
        let model_updates = ModelUpdates::new();
        let model = Model::new((0.0, 0.0, 0.0).into(), self.angle);
        model_updates.update(self.mesh_id, self.model_id, &model);
        self.renderer.mesh_manager().update_models(model_updates);
        self.angle += 0.01;

        self.renderer.render(device, queue, texture, &self.camera)
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}
