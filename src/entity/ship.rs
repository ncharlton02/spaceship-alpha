use super::ModelComp;
use crate::block::{BlockId, Blocks};
use crate::graphics::Model;
use cgmath::Point2;
use specs::{Builder, Component, Entity, VecStorage, World, WorldExt};
use std::collections::HashMap;

#[derive(Component)]
#[storage(VecStorage)]
pub struct ShipComp {
    tiles: HashMap<Point2<i16>, Tile>,
}

pub fn create_ship(world: &mut World) {
    let (wall, wall_mesh) = {
        let blocks = world.fetch::<Blocks>();
        (blocks.wall, blocks.get_block(blocks.wall).mesh_id)
    };
    let mut tiles = HashMap::new();
    let pos = Point2::new(0, 0);
    let model = Model::new((0.0, 0.0, 0.0).into(), 0.0);
    let tile_entity = world
        .create_entity()
        .with(TileComp { root: pos })
        .with(ModelComp::new(wall_mesh, model))
        .build();
    tiles.insert(
        pos,
        Tile {
            block: wall,
            entity: tile_entity,
        },
    );

    world.create_entity().with(ShipComp { tiles }).build();
}

pub struct Tile {
    block: BlockId,
    entity: Entity,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct TileComp {
    root: Point2<i16>,
}
