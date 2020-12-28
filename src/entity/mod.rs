use crate::block::Blocks;
use crate::graphics::{MeshId, MeshManager, Model, ModelId};
use specs::{prelude::*, Component};

pub use ship::{ShipComp, Tile, TileComp};

pub mod ship;

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct ModelComp {
    pub mesh_id: MeshId,
    pub model: Model,
    model_id: Option<ModelId>,
}

impl ModelComp {
    pub fn new(mesh_id: MeshId, model: Model) -> ModelComp {
        Self {
            mesh_id,
            model,
            model_id: None,
        }
    }
}

pub struct ModelUpdateSystem {
    reader_id: ReaderId<ComponentEvent>,
    inserted: BitSet,
    modified: BitSet,
    removed: BitSet,
}

impl<'a> System<'a> for ModelUpdateSystem {
    type SystemData = (WriteExpect<'a, MeshManager>, WriteStorage<'a, ModelComp>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut mesh_manager, mut model) = data;

        {
            self.inserted.clear();
            let events = model.channel().read(&mut self.reader_id);
            for event in events {
                match event {
                    ComponentEvent::Inserted(id) => self.inserted.add(*id),
                    ComponentEvent::Modified(id) => self.modified.add(*id),
                    ComponentEvent::Removed(id) => self.removed.add(*id),
                };
            }
        }

        for (model, _) in (&mut model, &self.inserted).join() {
            model.model_id = Some(mesh_manager.new_model(model.mesh_id, &model.model));
        }

        for (model, _) in (&mut model, &self.modified)
            .join()
            .filter(|(model, _)| model.model_id.is_some())
        {
            mesh_manager.update_model(model.mesh_id, model.model_id.unwrap(), &model.model);
        }

        for (_, _) in (&mut model, &self.removed)
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
    pub fn new(mesh_manager: MeshManager, blocks: Blocks) -> Self {
        let mut world = World::new();
        world.register::<ModelComp>();
        world.register::<ShipComp>();
        world.register::<TileComp>();
        world.insert(mesh_manager);
        world.insert(blocks);

        let model_update_system = {
            let mut comps = world.write_storage::<ModelComp>();
            ModelUpdateSystem {
                reader_id: comps.register_reader(),
                inserted: BitSet::new(),
                modified: BitSet::new(),
                removed: BitSet::new(),
            }
        };

        let dispatcher = DispatcherBuilder::new()
            .with(model_update_system, "update_models", &[])
            .build();

        ship::create_ship(&mut world);
        ECS { world, dispatcher }
    }

    pub fn update(&mut self) {
        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }
}
