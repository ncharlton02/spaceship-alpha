use super::{
    physics::{Collider, ColliderShape, Hitbox, RigidBody},
    EcsUtils, Model, Transform,
};
use crate::graphics::{MeshId, MeshManager};
use cgmath::{prelude::*, Vector3};
use specs::{prelude::*, world::LazyBuilder, Component};

/// Stores miscellaneous meshes (these are usually entities)
pub struct ObjectMeshes {
    pub asteroid: MeshId,
    pub mining_missle: MeshId,
}

impl ObjectMeshes {
    pub fn load(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> ObjectMeshes {
        let mut load = |name: &str| mesh_manager.add(device, &crate::graphics::load_mesh(name));

        Self {
            asteroid: load("asteroid"),
            mining_missle: load("mining_missle"),
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
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Health(u32);

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
    type SystemData = (Entities<'a>, Write<'a, EcsUtils>, ReadStorage<'a, Health>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut ecs_utils, healths) = data;

        for (entity, health) in (&entities, &healths).join() {
            if health.health() == 0 {
                ecs_utils.mark_for_removal(entity);
            }
        }
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Asteroid;

impl Asteroid {
    pub const HEALTH: u32 = 180;
}

pub fn create_asteroid(world: &mut World) {
    let mesh = world.fetch::<ObjectMeshes>().asteroid;

    world
        .create_entity()
        .with(Transform::from_position(-5.0, -5.0, 5.0))
        .with(Model::new(mesh))
        .with(RigidBody {
            velocity: Vector3::new(1.0, 0.0, 0.0),
        })
        .with(Collider::new(
            Hitbox::with_shape(ColliderShape::Sphere(0.8)),
            Collider::ASTEROID,
            vec![Collider::SHIP, Collider::MISSLE],
        ))
        .with(Asteroid)
        .with(Health(Asteroid::HEALTH))
        .build();
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
        Write<'a, EcsUtils>,
        ReadStorage<'a, MiningMissle>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, RigidBody>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut ecs_utils, missles, transforms, mut rigid_bodies) = data;

        for (entity, missle) in (&entities, &missles).join() {
            if !entities.is_alive(missle.target) {
                ecs_utils.mark_for_removal(entity);
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
