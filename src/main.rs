use cgmath::{prelude::*, Point2, Point3, Vector3};
use entity::ModelComp;
use graphics::{Camera, Mesh, Model, ModelUpdates, Renderer};
use specs::{Builder, World, WorldExt};
use std::{collections::HashSet};
use winit::event;

pub const WIREFRAME_MODE: bool = true;

mod app;
mod entity;
mod graphics;

struct AppState {
    renderer: Renderer,
    camera: Camera,
    world: World,
    keys: Keys,
}

impl AppState {
    fn update_camera(&mut self) {
        let rotate_speed = 0.01;
        let move_speed = 0.04;

        if self.keys.is_key_down(event::VirtualKeyCode::Q) {
            self.camera.yaw += rotate_speed;
        } else if self.keys.is_key_down(event::VirtualKeyCode::E) {
            self.camera.yaw -= rotate_speed;
        }

        let forward_power = if self.keys.is_key_down(event::VirtualKeyCode::W) {
            1.0
        } else if self.keys.is_key_down(event::VirtualKeyCode::S) {
            -1.0
        } else {
            0.0
        };
        let side_power = if self.keys.is_key_down(event::VirtualKeyCode::D) {
            -1.0
        } else if self.keys.is_key_down(event::VirtualKeyCode::A) {
            1.0
        } else {
            0.0
        };

        let (yaw_sin, yaw_cos) = self.camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, yaw_sin, 0.0).normalize() * forward_power * move_speed;
        let side = Vector3::new(-yaw_sin, yaw_cos, 0.0).normalize() * side_power * move_speed;
        self.camera.position += forward + side;
    }
}

impl app::Application for AppState {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, _: &wgpu::Queue) -> Self {
        let mut renderer = Renderer::new(device, &swapchain);
        let camera = Camera {
            position: (-3.0, 0.0, 3.0).into(),
            yaw: 0.0,
            pitch: -1.0,
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

        let keys = Keys(HashSet::new());

        AppState {
            renderer,
            camera,
            world,
            keys,
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

    fn key_event(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        match state {
            event::ElementState::Pressed => self.keys.0.insert(key),
            event::ElementState::Released => self.keys.0.remove(&key),
        };
    }

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, _: Point2<f32>) {}

    fn click_event(&mut self, _: event::MouseButton, _: event::ElementState, _: Point2<f32>) {}

    fn fixed_update(&mut self, _: &wgpu::Device, _: &wgpu::Queue) {
        self.update_camera();

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

struct Keys(HashSet<event::VirtualKeyCode>);

impl Keys {
    fn is_key_down(&self, key: event::VirtualKeyCode) -> bool {
        self.0.contains(&key)
    }
}
