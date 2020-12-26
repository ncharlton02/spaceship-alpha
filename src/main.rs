use cgmath::{prelude::*, Point2, Point3, Vector3};
use entity::ModelComp;
use graphics::{Camera, Mesh, MeshId, Model, ModelId, ModelUpdates, Renderer};
use specs::{Builder, Component, Dispatcher, World, WorldExt};
use winit::event;

pub const WIREFRAME_MODE: bool = true;

mod app;
mod entity;
mod graphics;

struct AppState {
    renderer: Renderer,
    camera: Camera,
    world: World,
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
        let model = Model::new((0.0, 0.0, 0.0).into(), 0.0);
        let model_id = renderer.mesh_manager().new_model(mesh_id, &model);

        let mut world = entity::initialize_ecs();
        world
            .create_entity()
            .with(ModelComp {
                mesh_id,
                model,
                model_id,
            })
            .build();

        AppState {
            renderer,
            camera,
            world,
        }
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

        self.world.insert(ModelUpdates::default());
        entity::update_ecs(&mut self.world);
        let model_updates = self.world.remove::<ModelUpdates>().unwrap();
        self.renderer.mesh_manager().update_models(model_updates);
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
