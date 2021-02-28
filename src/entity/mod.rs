use crate::graphics::{Camera, MeshId, MeshManager, ModelId};
use crate::{block::Blocks, floor::Floors};
use cgmath::{prelude::*, Matrix4, Point2, Point3, Quaternion, Vector3};
pub use input::{InputAction, InputManager};
pub use objects::ObjectMeshes;
pub use physics::{Collider, ColliderShape, Hitbox, RaycastWorld, RigidBody};
pub use ship::{BlockEntity, Ship, Tile};
use specs::{prelude::*, shred::Fetch, storage::MaskedStorage, Component};

pub mod input;
pub mod objects;
pub mod physics;
pub mod ship;

pub type SimpleStorage<'a, T> = Storage<'a, T, Fetch<'a, MaskedStorage<T>>>;

pub struct Model {
    pub mesh_id: MeshId,
    model_id: Option<ModelId>,
}

impl Component for Model {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Model {
    pub fn new(mesh_id: MeshId) -> Model {
        Self {
            mesh_id,
            model_id: None,
        }
    }
}

// TODO: Have models automatically deleted using flagged storage.
// Blocked By: https://github.com/amethyst/specs/issues/720
pub struct ModelUpdateSystem {
    transform_reader: ReaderId<ComponentEvent>,
    model_reader: ReaderId<ComponentEvent>,
    inserted: BitSet,
    modified: BitSet,
}

impl<'a> System<'a> for ModelUpdateSystem {
    type SystemData = (
        WriteExpect<'a, MeshManager>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Model>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut mesh_manager, transforms, mut models) = data;
        self.inserted.clear();
        self.modified.clear();

        for event in models.channel().read(&mut self.model_reader) {
            match event {
                ComponentEvent::Inserted(id) => self.inserted.add(*id),
                _ => false,
            };
        }

        for event in transforms.channel().read(&mut self.transform_reader) {
            match event {
                ComponentEvent::Modified(id) => self.modified.add(*id),
                _ => false,
            };
        }

        for (model, transform, _) in (&mut models, &transforms, &self.inserted).join() {
            model.model_id = Some(mesh_manager.new_model(model.mesh_id, transform.as_matrix()));
        }

        for (model, transform, _) in (&mut models, &transforms, &self.modified)
            .join()
            .filter(|(model, _, _)| model.model_id.is_some())
        {
            mesh_manager.update_model(
                model.mesh_id,
                model.model_id.unwrap(),
                transform.as_matrix(),
            );
        }
    }
}

pub struct ECS<'a> {
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> ECS<'a> {
    pub fn new(
        device: &wgpu::Device,
        mut mesh_manager: MeshManager,
        blocks: Blocks,
        floors: Floors,
        camera: Camera,
        window_size: WindowSize,
    ) -> Self {
        let meshes = ObjectMeshes::load(device, &mut mesh_manager);
        let hitbox_meshes = physics::HitboxMeshes::load(device, &mut mesh_manager);
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Ship>();
        world.register::<BlockEntity>();
        world.register::<Transform>();
        world.register::<RigidBody>();
        world.register::<Collider>();
        world.register::<Line>();
        world.insert(EcsUtils::default());
        world.insert(meshes);
        world.insert(hitbox_meshes);
        world.insert(mesh_manager);
        world.insert(blocks);
        world.insert(floors);
        world.insert(camera);
        world.insert(window_size);
        world.insert(RaycastWorld::new());
        world.insert(InputManager::new());
        objects::register_components(&mut world);
        crate::block::register_components(&mut world);

        let model_update_system = {
            let transform_reader = world.write_storage::<Transform>().register_reader();
            let model_reader = world.write_storage::<Model>().register_reader();
            ModelUpdateSystem {
                transform_reader,
                model_reader,
                inserted: BitSet::new(),
                modified: BitSet::new(),
            }
        };

        let mut dispatcher_builder = DispatcherBuilder::new()
            .with(input::CameraSystem, "camera_system", &[])
            .with(input::InputSystem, "input_system", &["camera_system"]);
        dispatcher_builder.add_barrier();
        crate::block::setup_systems(&mut dispatcher_builder);
        objects::setup_systems(&mut dispatcher_builder);
        dispatcher_builder.add_barrier();
        let dispatcher = dispatcher_builder
            .with(physics::PhysicsSystem, "physics_system", &[])
            .with(
                physics::RaycastSystem,
                "raycast_system",
                &["physics_system"],
            )
            .with(model_update_system, "update_models", &["raycast_system"])
            .build();

        ship::create_ship(&mut world);
        objects::create_asteroid(&mut world);

        ECS { world, dispatcher }
    }

    pub fn update(&mut self) {
        self.dispatcher.dispatch(&self.world);
        self.maintain();
    }

    pub fn maintain(&mut self) {
        {
            // TODO: Convert this to a system that is executed at the end of an update (pre-physics / rendering)
            let mut ecs_utils = self.world.fetch_mut::<EcsUtils>();
            let mut mesh_manager = self.world.fetch_mut::<MeshManager>();
            let mut raycast_world = self.world.fetch_mut::<RaycastWorld>();
            let hitbox_meshes = self.world.fetch::<physics::HitboxMeshes>();

            for entity in &ecs_utils.to_be_removed {
                if let Some(mut model) = self
                    .world
                    .write_component::<Model>()
                    .get_mut(*entity)
                    .filter(|model| model.model_id.is_some())
                {
                    mesh_manager.remove_model(model.mesh_id, model.model_id.unwrap());
                    model.model_id = None;
                }

                if let Some(mut collider) =
                    self.world.write_component::<Collider>().get_mut(*entity)
                {
                    raycast_world.remove(&mut collider, &mut mesh_manager, &hitbox_meshes);
                }

                self.world
                    .entities()
                    .delete(*entity)
                    .expect("Unable to delete entity marked for removal");
            }
            ecs_utils.to_be_removed.clear();
        }

        self.world.maintain();
    }

    #[allow(dead_code)]
    pub fn mark_for_removal(&mut self, entity: Entity) {
        self.world
            .get_mut::<EcsUtils>()
            .unwrap()
            .mark_for_removal(entity);
    }

    pub fn get_resource_mut<T: 'static + Sync + Send>(&self) -> specs::shred::FetchMut<T> {
        self.world.write_resource::<T>()
    }

    pub fn get_resource<T: 'static + Sync + Send>(&self) -> specs::shred::Fetch<T> {
        self.world.read_resource::<T>()
    }
}

#[derive(Default)]
pub struct EcsUtils {
    to_be_removed: Vec<Entity>,
}

impl EcsUtils {
    /// Marks an entity to be removed at the end of the update.
    /// This should be used over world.delete() because this will delete
    /// the model from the renderer
    #[allow(dead_code)]
    pub fn mark_for_removal(&mut self, entity: Entity) {
        if !self.to_be_removed.contains(&entity) {
            self.to_be_removed.push(entity);
        }
    }
}

/// Represents an entity's position, rotation, and scale within space.
#[derive(Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Point3<f32>,
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Transform {
    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            scale: Point3::new(1.0, 1.0, 1.0),
            rotation: Quaternion::from_angle_z(cgmath::Rad(0.0)),
        }
    }

    pub fn set_rotation_z(&mut self, theta: f32) {
        self.rotation = Quaternion::from_angle_z(cgmath::Rad(theta));
    }

    fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

/// TODO: Create seperate lines for component and GPU
/// and then add a 'visible' flag here
#[derive(Clone, Copy, Component)]
#[storage(HashMapStorage)]
pub struct Line {
    pub pt: Vector3<f32>,
    pub pt2: Vector3<f32>,
    pub color: Vector3<f32>,
}

unsafe impl bytemuck::Pod for Line {}
unsafe impl bytemuck::Zeroable for Line {}

pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl WindowSize {
    fn as_point(&self) -> Point2<f32> {
        Point2::new(self.width, self.height)
    }
}
