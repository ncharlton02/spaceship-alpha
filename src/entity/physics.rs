use super::{SimpleStorage, ToBeRemoved, Transform};
use crate::graphics::{Mesh, MeshId, MeshManager, ModelId, Vertex};
use cgmath::{prelude::*, Matrix4, Point3, Vector3};
use nalgebra::{
    base::Vector3 as NVector3,
    geometry::Point3 as NPoint3,
    geometry::{Isometry3, Quaternion, Translation3, UnitQuaternion},
};
use ncollide3d::{
    pipeline::narrow_phase::ContactEvent,
    pipeline::object::{CollisionGroups, CollisionObjectSlabHandle},
    query::Ray,
    shape,
    world::CollisionWorld,
};
use specs::{prelude::*, Component};

#[derive(Component)]
#[storage(VecStorage)]
pub struct RigidBody {
    pub velocity: Vector3<f32>,
}

pub struct PhysicsSystem;

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, ToBeRemoved>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, RigidBody>,
        ReadStorage<'a, super::BlockEntity>,
        ReadStorage<'a, super::objects::Asteroid>,
        ReadStorage<'a, super::objects::MiningMissle>,
        WriteStorage<'a, super::Ship>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut to_be_removed,
            mut transforms,
            colliders,
            bodies,
            blocks,
            asteroids,
            missles,
            mut ships,
        ) = data;
        let mut world: CollisionWorld<f32, Entity> = CollisionWorld::new(0.02);
        let dt = 1.0 / 60.0;
        let contact_query = ncollide3d::pipeline::object::GeometricQueryType::Contacts(0.0, 0.0);

        // Update Rigid Bodies
        for (transform, body) in (&mut transforms, &bodies).join() {
            transform.position += body.velocity * dt;
        }

        // Setup Collision
        for (entity, transform, collider) in (&entities, &transforms, &colliders).join() {
            let position = to_nalgebra_pos(&transform, &collider.hitbox.offset);
            let shape = collider.hitbox.as_shape_handle();
            let mut group = CollisionGroups::new()
                .with_membership(&[collider.group])
                .with_whitelist(&collider.whitelist);
            group.disable_self_interaction();

            world.add(position, shape, group, contact_query, entity);
        }
        // crate::print_time("PhysicsStart");
        world.update();
        // crate::print_time("PhysicsEnd");

        //Process Collisions
        fn has_component<T: Component>(
            e1: Entity,
            e2: Entity,
            component: &SimpleStorage<'_, T>,
        ) -> bool {
            component.contains(e1) || component.contains(e2)
        }

        for event in world.contact_events() {
            match event {
                ContactEvent::Started(h1, h2) => {
                    let entity1 = *world.collision_object(*h1).unwrap().data();
                    let entity2 = *world.collision_object(*h2).unwrap().data();

                    if has_component(entity1, entity2, &blocks) {
                        if asteroids.contains(entity1) {
                            to_be_removed.add(entity1);
                        } else if asteroids.contains(entity2) {
                            to_be_removed.add(entity2);
                        }

                        let block_entity = if blocks.contains(entity1) {
                            entity1
                        } else {
                            entity2
                        };
                        let ship_entity = blocks.get(block_entity).unwrap().ship;
                        let ship = ships.get_mut(ship_entity).unwrap();
                        ship.heat += 20.0;
                    }

                    if has_component(entity1, entity2, &missles)
                        && has_component(entity1, entity2, &asteroids)
                    {
                        to_be_removed.add(entity1);
                        to_be_removed.add(entity2);
                    }
                }
                ContactEvent::Stopped(_, _) => {}
            }
        }
    }
}

fn to_nalgebra_pos(transform: &Transform, offset: &Vector3<f32>) -> Isometry3<f32> {
    let translation = Translation3::new(
        transform.position.x + offset.x,
        transform.position.y + offset.y,
        transform.position.z + offset.z,
    );
    let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
        transform.rotation.s,
        transform.rotation.v.x,
        transform.rotation.v.y,
        transform.rotation.v.z,
    ));

    Isometry3::from_parts(translation, rotation)
}

#[derive(Clone)]
pub struct Hitbox {
    pub shape: ColliderShape,
    pub offset: Vector3<f32>,
}

impl Hitbox {
    pub fn new(shape: ColliderShape, offset: Vector3<f32>) -> Hitbox {
        Hitbox { shape, offset }
    }

    pub fn with_shape(shape: ColliderShape) -> Self {
        Hitbox::new(shape, Vector3::zero())
    }

    pub fn as_shape_handle(&self) -> shape::ShapeHandle<f32> {
        use ncollide3d::shape::*;

        match self.shape {
            ColliderShape::Cuboid(size) => {
                // NCollide Wants half-extents but we want to use the full size of the box
                ShapeHandle::new(Cuboid::new(NVector3::new(
                    size.x / 2.0,
                    size.y / 2.0,
                    size.z / 2.0,
                )))
            }
            ColliderShape::Sphere(radius) => ShapeHandle::new(Ball::new(radius)),
        }
    }

    pub fn to_hitbox_model(&self, transform: &Transform) -> Matrix4<f32> {
        let mut hb_transform = transform.clone();
        hb_transform.position += self.offset;

        match self.shape {
            ColliderShape::Cuboid(size) => hb_transform.scale = size,
            ColliderShape::Sphere(radius) => {
                hb_transform.scale = Vector3::new(radius, radius, radius);
            }
        }

        hb_transform.scale.x *= transform.scale.x;
        hb_transform.scale.y *= transform.scale.z;
        hb_transform.scale.z *= transform.scale.y;

        hb_transform.as_matrix()
    }

    pub fn to_hitbox_mesh(&self, meshes: &HitboxMeshes) -> MeshId {
        match self.shape {
            ColliderShape::Cuboid(_) => meshes.unit_cube,
            ColliderShape::Sphere(_) => meshes.unit_sphere,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// The Full Size of the Box
    Cuboid(Vector3<f32>),
    Sphere(f32),
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub hitbox: Hitbox,
    pub group: usize,
    pub whitelist: Vec<usize>,
    raycast_id: Option<CollisionObjectSlabHandle>,
    model_id: Option<ModelId>,
}

impl Collider {
    /// Internal Use Only. Used to prevent Raycast colliders
    /// from self interacting.
    const RAYCAST: usize = 0;
    pub const ASTEROID: usize = 1;
    pub const SHIP: usize = 2;
    pub const MISSLE: usize = 3;

    pub fn new(hitbox: Hitbox, group: usize, whitelist: Vec<usize>) -> Self {
        Self {
            hitbox,
            group,
            whitelist,
            raycast_id: None,
            model_id: None,
        }
    }
}

pub struct RaycastWorld(CollisionWorld<f32, Entity>);

impl RaycastWorld {
    pub fn new() -> Self {
        Self(CollisionWorld::new(0.02))
    }

    /// Note: If the whitelist is empty,
    /// then the whitelist is set to ALL groups.
    pub fn raycast(
        &self,
        whitelist: Vec<usize>,
        near: Vector3<f32>,
        far: Vector3<f32>,
    ) -> Option<Entity> {
        let origin = NPoint3::new(near.x, near.y, near.z);
        let dir = NVector3::new(far.x - near.x, far.y - near.y, far.z - near.z).normalize();
        let ray = Ray::new(origin, dir);
        let toi = 500.0; //TODO: Decide a good time of impact (currently just a big number)
        let mut groups = CollisionGroups::new();

        if !whitelist.is_empty() {
            groups.set_whitelist(&whitelist);
        }

        self.0
            .first_interference_with_ray(&ray, toi, &groups)
            .map(|result| *result.co.data())
    }
}

pub struct RaycastSystem;

impl<'a> System<'a> for RaycastSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, RaycastWorld>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Collider>,
        WriteExpect<'a, MeshManager>,
        ReadExpect<'a, HitboxMeshes>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut world, transforms, mut colliders, mut meshes, hitbox_meshes) = data;
        let world = &mut world.0;
        let contact_query = ncollide3d::pipeline::object::GeometricQueryType::Contacts(0.8, 0.8);

        for (entity, transform, mut colliders) in
            (&entities, &transforms, &mut colliders.restrict_mut()).join()
        {
            // Safety: We can use get_unchecked here because we know the entity is alive
            // from joining the Entities resource
            let collider = colliders.get_unchecked();
            let position = to_nalgebra_pos(&transform, &collider.hitbox.offset);
            let hitbox_mesh = collider.hitbox.to_hitbox_mesh(&hitbox_meshes);
            let hitbox_matrix = collider.hitbox.to_hitbox_model(&transform);

            if let (Some(id), Some(model)) = (collider.raycast_id, collider.model_id) {
                let collider_object = world
                    .get_mut(id)
                    .expect("Raycast ID does not exist in collision world!");
                collider_object.set_position(position);
                collider_object.set_shape(collider.hitbox.as_shape_handle());
                meshes.update_model(hitbox_mesh, model, hitbox_matrix);
            } else {
                let collider = colliders.get_mut_unchecked();
                let shape = collider.hitbox.as_shape_handle();
                let group = ncollide3d::pipeline::object::CollisionGroups::new()
                    .with_membership(&[collider.group])
                    .with_whitelist(&[Collider::RAYCAST]);

                collider.raycast_id =
                    Some(world.add(position, shape, group, contact_query, entity).0);

                // TODO: Rendering happens in the raycast update system? This either should be renamed
                // or needs to happen in a different system.
                collider.model_id = Some(meshes.new_model(hitbox_mesh, hitbox_matrix));
            }
        }
        world.update();
    }
}

pub struct RemoveRaycastColliderSystem;

impl<'a> System<'a> for RemoveRaycastColliderSystem {
    type SystemData = (
        Read<'a, ToBeRemoved>,
        WriteExpect<'a, RaycastWorld>,
        WriteExpect<'a, MeshManager>,
        ReadExpect<'a, HitboxMeshes>,
        WriteStorage<'a, Collider>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (to_be_removed, mut raycast_world, mut mesh_manager, hitbox_meshes, mut colliders) =
            data;

        for (collider, _) in (&mut colliders, to_be_removed.bitset()).join() {
            if let Some(id) = collider.raycast_id {
                raycast_world.0.remove(&[id]);
                collider.raycast_id = None;
            }

            if let Some(id) = collider.model_id {
                mesh_manager.remove_model(collider.hitbox.to_hitbox_mesh(&hitbox_meshes), id);
                collider.model_id = None;
            }
        }
    }
}

pub struct HitboxMeshes {
    pub unit_cube: MeshId,
    pub unit_sphere: MeshId,
}

impl HitboxMeshes {
    pub fn load(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> Self {
        let mut register_mesh = |mesh: &Mesh| {
            let id = mesh_manager.add(device, mesh);
            mesh_manager.set_mesh_visisble(id, crate::RENDER_HITBOXES);
            id
        };
        let unit_cube = Mesh::rectangular_prism(1.0, 1.0, 1.0, Point3::new(1.0, 0.0, 0.0));

        Self {
            unit_cube: register_mesh(&unit_cube),
            unit_sphere: register_mesh(&create_sphere_mesh()),
        }
    }
}

fn create_sphere_mesh() -> Mesh {
    let color = Point3::new(1.0, 0.0, 0.0);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let slices = 10;
    let chunks = 10;

    for slice in 0..slices + 1 {
        let slice_index = (chunks + 1) * slice;
        let next_slice_index = (chunks + 1) * (slice + 1);
        let (z_sin, z_cos) =
            ((crate::PI / slices as f32 * slice as f32) - (crate::PI / 2.0)).sin_cos();

        for chunk in 0..chunks + 1 {
            let (xy_sin, xy_cos) = (crate::PI * 2.0 / chunks as f32 * chunk as f32).sin_cos();
            let pos = Point3::new(z_cos * xy_cos, z_cos * xy_sin, z_sin);
            vertices.push(Vertex {
                normal: Point3::from_vec(pos.to_vec().normalize()),
                color,
                pos,
            });

            indices.push(slice_index + chunk);
            indices.push(slice_index + chunk + 1);
            indices.push(next_slice_index + chunk);
            indices.push(next_slice_index + chunk);
            indices.push(slice_index + chunk + 1);
            indices.push(next_slice_index + chunk + 1);
        }
    }

    Mesh {
        name: "Unit Sphere".to_string(),
        vertices,
        indices,
    }
}
