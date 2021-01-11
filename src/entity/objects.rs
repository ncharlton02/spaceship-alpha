use super::{
    physics::{Collider, ColliderShape, RigidBody},
    Model, Transform,
};
use crate::graphics::{MeshId, MeshManager};
use cgmath::Vector3;
use specs::{prelude::*, Component};

/// Stores miscellaneous meshes (these are usually entities)
pub struct ObjectMeshes {
    pub asteroid: MeshId,
}

impl ObjectMeshes {
    pub fn load(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> ObjectMeshes {
        let mut load = |name: &str| mesh_manager.add(device, &crate::graphics::load_mesh(name));

        Self {
            asteroid: load("asteroid"),
        }
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct AsteroidMarker;

pub fn create_asteroid(world: &mut World) {
    let asteroid = world.fetch::<ObjectMeshes>().asteroid;

    world
        .create_entity()
        .with(Transform::from_position(0.0, 0.0, 4.0))
        .with(Model::new(asteroid))
        .with(RigidBody {
            velocity: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Collider {
            shape: ColliderShape::Sphere(0.7),
            group: Collider::ASTEROID,
            whitelist: vec![Collider::SHIP, Collider::RAY],
        })
        .with(AsteroidMarker)
        .build();
}
