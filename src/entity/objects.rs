use super::{
    physics::{Collider, ColliderShape, Hitbox, RigidBody},
    Model, ToBeRemoved, Transform,
};
use crate::graphics::{MeshId, MeshManager};
use crate::item::{GameItem, Inventory};
use cgmath::{prelude::*, Vector3};
use specs::{prelude::*, world::LazyBuilder, Component};
use std::collections::HashMap;

/// Stores miscellaneous meshes (these are usually entities)
pub struct ObjectMeshes {
    pub asteroids: HashMap<GameItem, MeshId>,
    pub mining_missle: MeshId,
}

impl ObjectMeshes {
    pub fn load(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> ObjectMeshes {
        let asteroid_base = crate::graphics::load_mesh("asteroid");

        let asteroids: HashMap<GameItem, MeshId> = GameItem::asteroid_info()
            .iter()
            .map(|(item, color)| {
                let mut mesh = asteroid_base.clone();
                mesh.recolor(*color);
                (*item, mesh_manager.add(device, &mesh))
            })
            .collect();

        Self {
            asteroids,
            mining_missle: mesh_manager.add(device, &crate::graphics::load_mesh("mining_missle")),
        }
    }
}

pub fn register_components(world: &mut World) {
    world.register::<Asteroid>();
    world.register::<Health>();
    world.register::<MiningMissle>();
}

pub fn setup_systems(builder: &mut DispatcherBuilder) {
    builder.add(MiningMissleSystem, "", &[]);
    builder.add(NoMoreHealthSystem, "", &[]);
    builder.add(AsteroidShrinkSystem, "", &[]);
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Health(pub u32);

impl Health {
    pub fn damage(&mut self, amount: u32) {
        self.0 -= amount.min(self.0);
    }

    pub fn health(&self) -> u32 {
        self.0
    }
}

pub struct NoMoreHealthSystem;

impl<'a> System<'a> for NoMoreHealthSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, ToBeRemoved>,
        ReadStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut to_be_removed, healths) = data;

        for (entity, health) in (&entities, &healths).join() {
            if health.health() == 0 {
                to_be_removed.add(entity);
            }
        }
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Asteroid(pub GameItem);

impl Asteroid {
    pub const HEALTH: u32 = 180;
    pub const COLLIDER_RADIUS: f32 = 0.8;
    pub const VELOCITY: f32 = 1.0;
}

pub struct AsteroidShrinkSystem;

impl<'a> System<'a> for AsteroidShrinkSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Asteroid>,
        ReadStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut transforms, asteroids, healths) = data;

        for (transform, _, health) in (&mut transforms, &asteroids, &healths).join() {
            let scale = 0.5 + (health.health() as f32 / Asteroid::HEALTH as f32) / 2.0;
            transform.scale = Vector3::new(scale, scale, scale);
        }
    }
}

pub struct AsteroidMinedSystem;

impl<'a> System<'a> for AsteroidMinedSystem {
    type SystemData = (
        Read<'a, ToBeRemoved>,
        WriteExpect<'a, Inventory>,
        ReadStorage<'a, Asteroid>,
        ReadStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (to_be_removed, mut inventory, asteroids, healths) = data;

        for (_, asteroid, health) in (to_be_removed.bitset(), &asteroids, &healths).join() {
            if health.health() == 0 {
                // Need to make sure it was actually mined (and not just removed)
                inventory.add(asteroid.0, 5);
            }
        }
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct MiningMissle {
    target: Entity,
}

impl MiningMissle {
    const SPEED: f32 = 6.5;
}

pub fn build_mining_missle(
    meshes: &ObjectMeshes,
    builder: LazyBuilder,
    target: Entity,
    pos: Vector3<f32>,
) {
    builder
        .with(Transform::from_position(pos.x, pos.y, pos.z))
        .with(Model::new(meshes.mining_missle))
        .with(RigidBody {
            velocity: Vector3::new(0.0, 0.0, MiningMissle::SPEED),
        })
        .with(Collider::new(
            Hitbox::with_shape(ColliderShape::Sphere(0.2)),
            Collider::MISSLE,
            vec![Collider::ASTEROID],
        ))
        .with(MiningMissle { target })
        .build();
}

struct MiningMissleSystem;

impl<'a> System<'a> for MiningMissleSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, ToBeRemoved>,
        ReadStorage<'a, MiningMissle>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, RigidBody>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut to_be_removed, missles, transforms, mut rigid_bodies) = data;

        for (entity, missle) in (&entities, &missles).join() {
            if !entities.is_alive(missle.target) {
                to_be_removed.add(entity);
                continue;
            }

            let missle_pos = transforms.get(entity).unwrap().position;
            let target_pos = transforms.get(missle.target).unwrap().position;

            if missle_pos.z >= target_pos.z {
                rigid_bodies.get_mut(entity).unwrap().velocity = Vector3::new(
                    target_pos.x - missle_pos.x,
                    target_pos.y - missle_pos.y,
                    0.0,
                )
                .normalize()
                    * MiningMissle::SPEED;
            } else if missle_pos.z >= target_pos.z - 1.0 {
                let z_factor = 1.0 - (target_pos.z - missle_pos.z).powf(2.0);
                let mut velocity = Vector3::new(
                    target_pos.x - missle_pos.x,
                    target_pos.y - missle_pos.y,
                    0.0,
                )
                .normalize()
                    * z_factor
                    * MiningMissle::SPEED;
                velocity.z = MiningMissle::SPEED * (1.0 - z_factor).max(0.4);
                rigid_bodies.get_mut(entity).unwrap().velocity = velocity;
            }
        }
    }
}
