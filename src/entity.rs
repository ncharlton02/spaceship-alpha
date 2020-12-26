use crate::graphics::{MeshId, Model, ModelId, ModelUpdates};
use specs::{
    Component, DispatcherBuilder, Join, Read, ReadStorage, System, VecStorage, World,
    WorldExt,
};

#[derive(Component)]
#[storage(VecStorage)]
pub struct ModelComp {
    pub mesh_id: MeshId,
    pub model_id: ModelId,
    pub model: Model,
}

/// Updates a model every fixed update
pub struct ModelUpdateSystem;

impl<'a> System<'a> for ModelUpdateSystem {
    type SystemData = (Read<'a, ModelUpdates>, ReadStorage<'a, ModelComp>);

    fn run(&mut self, data: Self::SystemData) {
        let (updates, model) = data;

        for model in model.join() {
            updates.update(model.mesh_id, model.model_id, &model.model);
        }
    }
}

pub fn initialize_ecs() -> World {
    let mut world = World::new();
    world.register::<ModelComp>();

    world
}

pub fn update_ecs(world: &mut World) {
    let mut dispatcher = DispatcherBuilder::new()
        .with(ModelUpdateSystem, "update_models", &[])
        .build();

    dispatcher.dispatch(world);
    world.maintain();
}
