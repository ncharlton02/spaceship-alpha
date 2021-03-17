use super::ship::{self, BlockEntity, BuildAction, Ship, Tile};
use super::{Collider, Model, RaycastWorld, ToBeRemoved, Transform, WindowSize, ECS};
use crate::block::{BlockId, Blocks};
use crate::graphics::Camera;
use cgmath::{InnerSpace, Point2, Vector3};
use specs::prelude::*;
use std::collections::HashSet;
use winit::event;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputAction {
    Mining,
    Laser,
    Build(BlockId),
    None,
}

impl InputAction {
    pub fn on_click(&self, ecs: &mut ECS) {
        match self {
            Self::Build(block_id) => {
                let block_entity = {
                    let input_manager = ecs.get_resource::<InputManager>();
                    let block_entities = ecs.get_component::<BlockEntity>();
                    input_manager
                        .target
                        .and_then(|entity| block_entities.get(entity))
                        .map(|x| x.clone())
                };

                if let Some(block_entity) = block_entity {
                    let ship = block_entity.ship;
                    let pos = block_entity.root;

                    ship::execute_build_actions(
                        &mut ecs.world,
                        ship,
                        &[BuildAction::BuildBlock(pos, *block_id)],
                        false,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn display_name(&self, ecs: &ECS) -> String {
        match self {
            InputAction::Build(block_id) => {
                let blocks = ecs.get_resource::<Blocks>();
                let block = blocks.get_block(*block_id);

                format!("Build {}", block.type_name)
            }
            _ => format!("{:?}", self),
        }
    }
}

pub enum MouseState {
    Released,
    JustReleased,
    JustPressed,
    Pressed,
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

pub fn setup_systems(builder: &mut DispatcherBuilder) {
    builder.add(CameraSystem, "camera_system", &[]);
    builder.add(InputSystem, "input_system", &["camera_system"]);
    // TODO - There should be a way to automatically change
    // the rendering system based on the current input action
    builder.add(
        BuildRenderSystem(None, None),
        "build_render_system",
        &["input_system"],
    );
}

pub struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (ReadExpect<'a, InputManager>, WriteExpect<'a, Camera>);

    fn run(&mut self, data: Self::SystemData) {
        let (input, mut camera) = data;
        let rotate_speed = 0.015;
        let move_speed = 0.12;

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
            InputAction::Build(_) => Some(vec![Collider::SHIP]),
            _ => None,
        }
        .and_then(|collider| raycaster.raycast(collider, near, far));
    }
}

pub struct BuildRenderSystem(Option<Entity>, Option<BlockId>);

impl<'a> System<'a> for BuildRenderSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        WriteExpect<'a, ToBeRemoved>,
        ReadExpect<'a, InputManager>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, BlockEntity>,
        ReadExpect<'a, Blocks>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            lazy_update,
            mut to_be_removed,
            input_manager,
            mut transforms,
            block_entities,
            blocks,
        ) = data;

        let create_transform = |target: Option<Entity>| {
            if let Some(block_entity) = target.and_then(|target| block_entities.get(target).clone())
            {
                let block = blocks.get_block(block_entity.block_id);

                let mut transform = Transform::from_position(
                    block_entity.root.x as f32,
                    block_entity.root.y as f32,
                    block.height,
                );
                transform.scale = Vector3::new(0.5, 0.5, 0.8);
                transform
            } else {
                // TODO: Better way of setting invisible
                let mut transform = Transform::from_position(0.0, 0.0, 0.0);
                transform.scale = Vector3::new(0.0, 0.0, 0.0);
                transform
            }
        };

        match input_manager.action {
            InputAction::Build(block_id) if self.0.is_none() => {
                let transform = create_transform(input_manager.target);

                self.0 = Some(
                    lazy_update
                        .create_entity(&entities)
                        .with(transform)
                        .with(Model::new(blocks.get_block(block_id).mesh_id))
                        .build(),
                );
                self.1 = Some(block_id);
            }
            InputAction::Build(block_id) if self.1.is_some() && self.1 != Some(block_id) => {
                // TODO: (Cleanup) This block is a copy of the None block :(
                to_be_removed.add(self.0.unwrap());
                self.0 = None;
                self.1 = None;
            }
            InputAction::Build(_) => {
                if let Some(transform) = transforms.get_mut(self.0.unwrap()) {
                    *transform = create_transform(input_manager.target);
                }
            }
            InputAction::None if self.0.is_some() => {
                // Remove Entity
                to_be_removed.add(self.0.unwrap());
                self.0 = None;
                self.1 = None;
            }
            _ => {}
        }
    }
}
