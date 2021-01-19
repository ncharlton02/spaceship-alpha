use super::{EcsUtils, SimpleStorage, Transform};
use cgmath::Vector3;
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
        Write<'a, EcsUtils>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, RigidBody>,
        ReadStorage<'a, super::BlockEntity>,
        ReadStorage<'a, super::objects::AsteroidMarker>,
        ReadStorage<'a, super::objects::MiningMissle>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut ecs_utils,
            mut transforms,
            colliders,
            bodies,
            blocks,
            asteroids,
            missles,
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
            let position = to_nalgebra_pos(&transform);
            let shape = collider.shape.as_shape_handle();
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
                            ecs_utils.mark_for_removal(entity1);
                        } else if asteroids.contains(entity2) {
                            ecs_utils.mark_for_removal(entity2);
                        }
                    }

                    if has_component(entity1, entity2, &missles)
                        && has_component(entity1, entity2, &asteroids)
                    {
                        ecs_utils.mark_for_removal(entity1);
                        ecs_utils.mark_for_removal(entity2);
                    }
                }
                ContactEvent::Stopped(_, _) => {}
            }
        }
    }
}

fn to_nalgebra_pos(transform: &Transform) -> Isometry3<f32> {
    let translation = Translation3::new(
        transform.position.x,
        transform.position.y,
        transform.position.z,
    );
    let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
        transform.rotation.s,
        transform.rotation.v.x,
        transform.rotation.v.y,
        transform.rotation.v.z,
    ));

    Isometry3::from_parts(translation, rotation)
}

pub enum ColliderShape {
    /// The Full Size of the Box
    Cuboid(Vector3<f32>),
    Sphere(f32),
}

impl ColliderShape {
    pub fn as_shape_handle(&self) -> shape::ShapeHandle<f32> {
        use ncollide3d::shape::*;

        match self {
            ColliderShape::Cuboid(size) => {
                // NCollide Wants half-extents but we want to use the full size of the box
                ShapeHandle::new(Cuboid::new(NVector3::new(
                    size.x / 2.0,
                    size.y / 2.0,
                    size.z / 2.0,
                )))
            }
            ColliderShape::Sphere(radius) => ShapeHandle::new(Ball::new(*radius)),
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub shape: ColliderShape,
    pub group: usize,
    pub whitelist: Vec<usize>,
    raycast_id: Option<CollisionObjectSlabHandle>,
}

impl Collider {
    /// Internal Use Only. Used to prevent Raycast colliders
    /// from self interacting.
    const RAYCAST: usize = 0;
    pub const ASTEROID: usize = 1;
    pub const SHIP: usize = 2;
    pub const MISSLE: usize = 3;

    pub fn new(shape: ColliderShape, group: usize, whitelist: Vec<usize>) -> Self {
        Self {
            shape,
            group,
            whitelist,
            raycast_id: None,
        }
    }
}

pub struct RaycastWorld(CollisionWorld<f32, Entity>);

impl RaycastWorld {
    pub fn new() -> Self {
        Self(CollisionWorld::new(0.02))
    }

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
        let groups = CollisionGroups::new().with_whitelist(&whitelist);

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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut world, transforms, mut colliders) = data;
        let world = &mut world.0;
        let contact_query = ncollide3d::pipeline::object::GeometricQueryType::Contacts(0.8, 0.8);

        for (entity, transform, mut colliders) in
            (&entities, &transforms, &mut colliders.restrict_mut()).join()
        {
            let position = to_nalgebra_pos(&transform);

            // Safety: We can use get_unchecked here because we know the entity is alive
            // from joining the Entities resource
            if let Some(id) = colliders.get_unchecked().raycast_id {
                let collider_object = world
                    .get_mut(id)
                    .expect("Raycast ID does not exist in collision world!");
                collider_object.set_position(position);
            } else {
                let collider = colliders.get_mut_unchecked();
                let shape = collider.shape.as_shape_handle();
                let group = ncollide3d::pipeline::object::CollisionGroups::new()
                    .with_membership(&[collider.group])
                    .with_whitelist(&[Collider::RAYCAST]);

                collider.raycast_id =
                    Some(world.add(position, shape, group, contact_query, entity).0);
            }
        }
        world.update();
    }
}
