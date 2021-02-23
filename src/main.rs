#[macro_use]
extern crate lazy_static;

use cgmath::{prelude::*, Point2, Vector3};
use entity::{Collider, ECS};
use graphics::{Camera, MeshManager, Renderer};
use specs::prelude::*;
use std::collections::HashSet;
use ui::Ui;
use winit::event;

pub const WIREFRAME_MODE: bool = false;
pub const RENDER_HITBOXES: bool = false;
pub const RENDER_BLOCKS: bool = true;
pub const MSAA_SAMPLE: u32 = 4; //TODO - determine this dynamically
pub const PI: f32 = std::f32::consts::PI;

mod app;
mod block;
mod entity;
mod floor;
mod graphics;
mod ui;

struct AppState {
    renderer: Renderer,
    camera: Camera,
    ecs: entity::ECS<'static>,
    keys: Keys,
    window_size: Point2<f32>,
    left_click: Option<Point2<f32>>,
    ui: Ui,
}

impl AppState {
    fn update_camera(&mut self) {
        let rotate_speed = 0.02;
        let move_speed = 0.16;

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
    fn init(
        swapchain: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut mesh_manager = MeshManager::new();
        let (renderer, ui_assets) = Renderer::new(device, queue, &swapchain);
        let blocks = block::load_blocks(device, &mut mesh_manager);
        let floors = floor::load_floors(device, &mut mesh_manager);
        let camera = Camera {
            position: (-12.0, 0.0, 12.0).into(),
            yaw: 0.0,
            pitch: -1.0,
            aspect: swapchain.width as f32 / swapchain.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        let ecs = ECS::new(device, mesh_manager, blocks, floors);
        let keys = Keys(HashSet::new());
        let window_size = Point2::new(swapchain.width as f32, swapchain.height as f32);
        let ui = Ui::new(ui_assets);
        queue.submit(None);

        AppState {
            renderer,
            camera,
            ecs,
            keys,
            window_size,
            ui,
            left_click: None,
        }
    }

    fn resize(
        &mut self,
        swapchain: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.camera.resize(swapchain);
        self.renderer.resize(device, queue, swapchain);
        self.window_size = Point2::new(swapchain.width as f32, swapchain.height as f32);
    }

    fn key_event(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        match state {
            event::ElementState::Pressed => self.keys.0.insert(key),
            event::ElementState::Released => self.keys.0.remove(&key),
        };
    }

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, new_pos: Point2<f32>) {
        if let Some(left_click) = &mut self.left_click {
            *left_click = new_pos;
        }
    }

    fn click_event(
        &mut self,
        button: event::MouseButton,
        state: event::ElementState,
        mut pt: Point2<f32>,
    ) {
        pt.y = self.window_size.y - pt.y;
        self.ui.on_click(button, state, pt);

        if button != event::MouseButton::Left {
            return;
        }

        match state {
            event::ElementState::Pressed => {
                self.left_click = Some(pt);
            }
            event::ElementState::Released => {
                self.left_click = None;
                self.ecs.set_input_action(InputAction::None);
            }
        }
    }

    fn fixed_update(&mut self, _: &wgpu::Device, _: &wgpu::Queue) {
        self.update_camera();
        self.ecs.update();

        let pt = if let Some(pt) = self.left_click {
            pt
        } else {
            return;
        };

        let near = self
            .camera
            .unproject(Vector3::new(pt.x, pt.y, 0.0), self.window_size);
        let far = self
            .camera
            .unproject(Vector3::new(pt.x, pt.y, 1.0), self.window_size);

        let action = match self.ecs.raycast(vec![Collider::ASTEROID], near, far) {
            Some(entity) => InputAction::Laser(entity),
            None => InputAction::None,
        };

        self.ecs.set_input_action(action);
    }

    fn render(
        &mut self,
        texture: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut lines = Vec::new();
        let lines_comps = self.ecs.world.read_component::<entity::Line>();
        let entities = self.ecs.world.fetch::<specs::world::EntitiesRes>();

        for (line, _) in (&lines_comps, &entities).join() {
            lines.push(*line);
        }

        let mut mesh_manager = self.ecs.world.fetch_mut::<MeshManager>();
        self.ui.update();
        self.ui.render(&mut self.renderer.ui_renderer.batch);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.renderer.render_world(
            device,
            queue,
            texture,
            &mut encoder,
            &self.camera,
            &mut mesh_manager,
            &lines,
        );

        self.renderer
            .render_ui(device, queue, texture, &mut encoder);
        queue.submit(Some(encoder.finish()));
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

#[allow(dead_code)]
pub fn print_time(title: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        % 1000;

    println!("{}: {}", title, time_ms);
}

pub enum InputAction {
    Mining(Entity),
    Laser(Entity),
    None,
}
