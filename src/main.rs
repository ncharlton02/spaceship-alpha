#[macro_use]
extern crate lazy_static;

use cgmath::Point2;
use entity::{InputManager, WindowSize, ECS};
use graphics::{Camera, MeshManager, Renderer};
use specs::prelude::*;
use ui::{Ui, UiAssets, NodeId};
use winit::event;
use std::time::{SystemTime, UNIX_EPOCH};

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
mod item;
mod ui;

struct AppState {
    renderer: Renderer,
    ecs: entity::ECS<'static>,
    ui: Ui,
    game_over: bool,
    ui_scene: NodeId,
    start_time: SystemTime,
}

impl app::Application for AppState {
    fn init(
        swapchain: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut mesh_manager = MeshManager::new();
        let mut renderer = Renderer::new(device, &swapchain);
        let blocks = block::load_blocks(device, &mut mesh_manager);
        let floors = floor::load_floors(device, &mut mesh_manager);
        let camera = Camera {
            position: (18.0, 8.0, 18.0).into(),
            yaw: PI,
            pitch: -1.3,
            aspect: swapchain.width as f32 / swapchain.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };
        let window_size = WindowSize {
            width: swapchain.width as f32,
            height: swapchain.height as f32,
        };

        let ecs = ECS::new(device, mesh_manager, blocks, floors, camera, window_size);
        let ui_assets = UiAssets::new(device, queue, &mut renderer.ui_renderer.batch.atlas);
        let mut ui = Ui::new(ui_assets);
        let ui_scene = ui::in_game::create_in_game_ui(&mut ui);
        let start_time = SystemTime::now();

        queue.submit(None);

        AppState {
            renderer,
            ecs,
            ui,
            ui_scene,
            start_time,
            game_over: false,
        }
    }

    fn resize(
        &mut self,
        swapchain: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.ecs.get_resource_mut::<Camera>().resize(swapchain);
        self.renderer.resize(device, queue, swapchain);

        let mut window_size = self.ecs.get_resource_mut::<WindowSize>();
        window_size.width = swapchain.width as f32;
        window_size.height = swapchain.height as f32;
    }

    fn key_event(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        self.ecs
            .get_resource_mut::<InputManager>()
            .keys
            .update(key, state);
    }

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, new_pos: Point2<f32>) {
        let window_size = self.ecs.get_resource::<WindowSize>();
        let new_pos = Point2::new(new_pos.x, window_size.height - new_pos.y);
        self.ecs.get_resource_mut::<InputManager>().mouse_pos = new_pos;
    }

    fn click_event(
        &mut self,
        button: event::MouseButton,
        state: event::ElementState,
        mut pt: Point2<f32>,
    ) {
        let pressed = state == event::ElementState::Pressed;
        pt.y = self.ecs.get_resource::<WindowSize>().height - pt.y;

        if !self.ui.on_click(button, state, pt) && button == event::MouseButton::Left {
            let input_action = {
                let mut input_manager = self.ecs.get_resource_mut::<InputManager>();
                input_manager.left_mb = pressed;
                input_manager.action
            };

            if pressed {
                input_action.on_click(&mut self.ecs);
            }
        }
    }

    fn fixed_update(&mut self, _: &wgpu::Device, _: &wgpu::Queue) {
        self.ui.update(&mut self.ecs);

        if !self.game_over {
            self.ecs.update();

            let player = self.ecs.player_ship;
            let ship_heat = self.ecs.get_component::<entity::Ship>().get(player).unwrap().heat;

            if ship_heat > entity::ship::MAX_HEAT {
                let time_elapsed = self.start_time.elapsed().unwrap().as_secs();
                self.game_over = true;
                // TODO: (BUG) the main menu must be added before the old one is removed??
                let main_menu = ui::end_game::create_end_game(&mut self.ui, time_elapsed);
                self.ui.remove_node(self.ui_scene);
                self.ui_scene = main_menu;
            }
        }
    }   

    fn render(
        &mut self,
        texture: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if !self.game_over {
            let mut lines = Vec::new();
            let lines_comps = self.ecs.world.read_component::<entity::Line>();
            let entities = self.ecs.get_resource::<specs::world::EntitiesRes>();
            let camera = self.ecs.get_resource::<Camera>();
            let mut mesh_manager = self.ecs.get_resource_mut::<MeshManager>();

            for (line, _) in (&lines_comps, &entities).join() {
                lines.push(*line);
            }

            self.renderer.render_world(
                queue,
                texture,
                &mut encoder,
                &camera,
                &mut mesh_manager,
                &lines,
            );
        }

        self.ui.render(&mut self.renderer.ui_renderer.batch);
        self.renderer.render_ui(queue, texture, &mut encoder, self.game_over);
        queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}

#[allow(dead_code)]
pub fn print_time(title: &str) {
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        % 1000;

    println!("{}: {}", title, time_ms);
}
