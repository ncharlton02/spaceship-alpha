use super::ModelComp;
use crate::block::{Block, BlockId, Blocks};
use crate::graphics::Model;
use cgmath::Point2;
use specs::{prelude::*, Component};
use std::collections::HashMap;

#[derive(Component)]
#[storage(VecStorage)]
pub struct ShipComp {
    tiles: HashMap<Point2<i16>, Tile>,
}

/// A system that will build a block at a point
pub struct ShipBuildSystem {
    positions: Vec<Point2<i16>>,
    block_id: BlockId,
}

impl<'a> System<'a> for ShipBuildSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Blocks>,
        WriteStorage<'a, ShipComp>,
        WriteStorage<'a, TileComp>,
        WriteStorage<'a, ModelComp>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut entities, blocks, mut ships, mut tiles, mut models) = data;
        let block = blocks.get_block(self.block_id);

        for pos in &self.positions {
            let model = Model::new((pos.x as f32, pos.y as f32, 0.0).into(), 0.0);
            let tile_entity = entities.create();
            let tile = Tile {
                block: block.id,
                entity: tile_entity,
            };
            tiles.insert(tile_entity, TileComp { root: *pos });
            models.insert(tile_entity, ModelComp::new(block.mesh_id, model));

            for ship in (&mut ships).join() {
                ship.tiles.insert(*pos, tile.clone());
            }

            println!("Built {} @ {:?}", block.type_name, pos);
        }
    }
}

pub fn create_ship(world: &mut World) {
    let tiles = HashMap::new();
    world.create_entity().with(ShipComp { tiles }).build();

    let mut positions = Vec::new();
    let size = 15;
    for x in 0..=size {
        for y in 0..=size {
            if x == 0 || y == 0 || x == size || y == size {
                positions.push(Point2::new(x, y));
            }
        }
    }

    let wall = world.fetch::<Blocks>().wall;
    let builder = ShipBuildSystem {
        positions,
        block_id: wall,
    }
    .run_now(&world);
}

#[derive(Clone)]
pub struct Tile {
    block: BlockId,
    entity: Entity,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct TileComp {
    root: Point2<i16>,
}
