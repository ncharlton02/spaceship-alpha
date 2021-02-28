use super::{Collider, RaycastWorld, WindowSize};
use crate::graphics::Camera;
use cgmath::{InnerSpace, Point2, Vector3};
use specs::prelude::*;
use std::collections::HashSet;
use winit::event;

#[derive(Debug, PartialEq, Eq)]
pub enum InputAction {
    Mining,
    Laser,
    None,
}

pub struct InputManager {
    pub action: InputAction,
    pub left_mb: bool,
    pub mouse_pos: Point2<f32>,
    pub keys: Keys,
    pub target: Option<Entity>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            action: InputAction::None,
            left_mb: false,
            mouse_pos: Point2::new(0.0, 0.0),
            target: None,
            keys: Keys(HashSet::new()),
        }
    }
}

pub struct Keys(HashSet<event::VirtualKeyCode>);

impl Keys {
    pub fn update(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        match state {
            event::ElementState::Pressed => self.0.insert(key),
            event::ElementState::Released => self.0.remove(&key),
        };
    }

    fn is_key_down(&self, key: event::VirtualKeyCode) -> bool {
        self.0.contains(&key)
    }
}

pub struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (ReadExpect<'a, InputManager>, WriteExpect<'a, Camera>);

    fn run(&mut self, data: Self::SystemData) {
        let (input, mut camera) = data;
        let rotate_speed = 0.02;
        let move_speed = 0.16;

        if input.keys.is_key_down(event::VirtualKeyCode::Q) {
            camera.yaw += rotate_speed;
        } else if input.keys.is_key_down(event::VirtualKeyCode::E) {
            camera.yaw -= rotate_speed;
        }

        let forward_power = if input.keys.is_key_down(event::VirtualKeyCode::W) {
            1.0
        } else if input.keys.is_key_down(event::VirtualKeyCode::S) {
            -1.0
        } else {
            0.0
        };
        let side_power = if input.keys.is_key_down(event::VirtualKeyCode::D) {
            -1.0
        } else if input.keys.is_key_down(event::VirtualKeyCode::A) {
            1.0
        } else {
            0.0
        };

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, yaw_sin, 0.0).normalize() * forward_power * move_speed;
        let side = Vector3::new(-yaw_sin, yaw_cos, 0.0).normalize() * side_power * move_speed;
        camera.position += forward + side;
    }
}

pub struct InputSystem;

impl<'a> System<'a> for InputSystem {
    type SystemData = (
        WriteExpect<'a, InputManager>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a, WindowSize>,
        ReadExpect<'a, RaycastWorld>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut input, camera, window_size, raycaster) = data;

        if !input.left_mb {
            input.target = None;
            return;
        }

        let near = camera.unproject(
            Vector3::new(input.mouse_pos.x, input.mouse_pos.y, 0.0),
            window_size.as_point(),
        );
        let far = camera.unproject(
            Vector3::new(input.mouse_pos.x, input.mouse_pos.y, 1.0),
            window_size.as_point(),
        );

        input.target = match input.action {
            InputAction::Mining | InputAction::Laser => Some(vec![Collider::ASTEROID]),
            _ => None,
        }
        .and_then(|collider| raycaster.raycast(vec![Collider::ASTEROID], near, far));
    }
}
