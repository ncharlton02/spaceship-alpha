use crate::graphics::{self, MeshId, MeshManager, ModelId};
use crate::{block::Blocks, floor::Floors};
use cgmath::{prelude::*, Matrix4, Point3, Quaternion, Vector3};
use specs::{prelude::*, shred::FetchMut, storage::MaskedStorage, Component};

pub use ship::{BlockEntity, Ship, Tile};

pub mod ship;

pub type SimpleStorage<'a, T> = Storage<'a, T, FetchMut<'a, MaskedStorage<T>>>;

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Model {
    pub mesh_id: MeshId,
    model_id: Option<ModelId>,
}

impl Model {
    pub fn new(mesh_id: MeshId) -> Model {
        Self {
            mesh_id,
            model_id: None,
        }
    }
}

pub struct ModelUpdateSystem {
    transform_reader: ReaderId<ComponentEvent>,
    model_reader: ReaderId<ComponentEvent>,
    inserted: BitSet,
    modified: BitSet,
    removed: BitSet,
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
        self.removed.clear();

        for event in models.channel().read(&mut self.model_reader) {
            match event {
                ComponentEvent::Inserted(id) => self.inserted.add(*id),
                ComponentEvent::Removed(id) => self.removed.add(*id),
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
        // TODO -> Filter Map?
        {
            mesh_manager.update_model(
                model.mesh_id,
                model.model_id.unwrap(),
                transform.as_matrix(),
            );
        }

        for (_, _) in (&mut models, &self.removed)
            .join()
            .filter(|(model, _)| model.model_id.is_some())
        {
            unimplemented!();
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
    ) -> Self {
        let meshes = MiscMeshes::load(device, &mut mesh_manager);
        let asteroid = meshes.asteroid;
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Ship>();
        world.register::<BlockEntity>();
        world.register::<Transform>();
        world.insert(meshes);
        world.insert(mesh_manager);
        world.insert(blocks);
        world.insert(floors);

        let model_update_system = {
            let transform_reader = world.write_storage::<Transform>().register_reader();
            let model_reader = world.write_storage::<Model>().register_reader();
            ModelUpdateSystem {
                transform_reader,
                model_reader,
                inserted: BitSet::new(),
                modified: BitSet::new(),
                removed: BitSet::new(),
            }
        };

        let dispatcher = DispatcherBuilder::new()
            .with(model_update_system, "update_models", &[])
            .build();

        ship::create_ship(&mut world);

        world
            .create_entity()
            .with(Transform::from_position(-5.0, 0.0, 3.0))
            .with(Model::new(asteroid))
            .build();

        ECS { world, dispatcher }
    }

    pub fn update(&mut self) {
        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }
}

/// Represents an entity's position, rotation, and scale within space.
#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Transform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Point3<f32>,
}

impl Transform {
    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            scale: Point3::new(1.0, 1.0, 1.0),
            rotation: Quaternion::from_angle_z(cgmath::Rad(0.0)),
        }
    }

    fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

/// Stores miscellaneous meshes (these are usually entities)
pub struct MiscMeshes {
    pub asteroid: MeshId,
}

impl MiscMeshes {
    pub fn load(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> MiscMeshes {
        let mut load = |name: &str| mesh_manager.add(device, &graphics::load_mesh(name));

        MiscMeshes {
            asteroid: load("asteroid"),
        }
    }
}
