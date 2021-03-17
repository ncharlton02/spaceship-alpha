use super::{Collider, Model, Transform};
use crate::block::{BlockId, Blocks};
use crate::floor::{Floor, Floors};
use crate::item::{GameItem, Inventory, ItemStack};
use cgmath::Point2;
use specs::{prelude::*, world::EntitiesRes, Component};
use std::collections::HashMap;

pub const MAX_HEAT: f32 = 100.0;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Ship {
    pub heat: f32,
    tiles: HashMap<Point2<i16>, Tile>,
}

#[derive(Clone, Debug)]
pub struct Tile {
    block: Option<Entity>,
    gadget: Option<Entity>,
    floor: Option<Entity>,
}

#[derive(Component, Clone)]
#[storage(DenseVecStorage)]
pub struct BlockEntity {
    pub ship: Entity,
    pub block_id: BlockId,
    pub root: Point2<i16>,
}

#[derive(Component, Clone)]
#[storage(DenseVecStorage)]
pub struct GadgetEntity {
    pub ship: Entity,
    pub block_id: BlockId,
}

pub fn register_components(world: &mut World) {
    world.register::<Ship>();
    world.register::<BlockEntity>();
    world.register::<GadgetEntity>();
}

pub enum BuildAction {
    BuildBlock(Point2<i16>, BlockId),
    RemoveBlock(Point2<i16>),
    BuildFloor(Point2<i16>, Floor),
    RemoveFloor(Point2<i16>),
}

pub fn execute_build_actions(
    world: &mut World,
    ship_entity: Entity,
    actions: &[BuildAction],
    ignore_price: bool,
) {
    let lazy_update = world.fetch::<LazyUpdate>();
    let entities = world.fetch::<EntitiesRes>();
    let mut ships = world.write_component::<Ship>();
    let mut inventory = world.fetch_mut::<Inventory>();
    let ship = ships.get_mut(ship_entity).unwrap();
    let blocks = world.fetch::<Blocks>();
    let block_entities = world.read_component::<BlockEntity>();

    for action in actions {
        match action {
            BuildAction::BuildBlock(pos, block_id) if blocks.get_block(*block_id).is_gadget => {
                let block = blocks.get_block(*block_id);

                if block.size.x > 1 || block.size.y > 1 {
                    unimplemented!("Multiblock gadgets not supported!");
                }

                let base = if let Some(block) = ship
                    .tiles
                    .get(pos)
                    .and_then(|tile| tile.block)
                    .and_then(|entity| block_entities.get(entity))
                    .map(|block_entity| blocks.get_block(block_entity.block_id))
                {
                    block
                } else {
                    continue;
                };

                if !ignore_price {
                    if block
                        .cost
                        .iter()
                        .filter(|cost| !inventory.has_enough_items(*cost))
                        .count()
                        > 0
                    {
                        break;
                    } else {
                        block
                            .cost
                            .iter()
                            .for_each(|cost| inventory.remove(cost.item, cost.amount));
                    }
                }

                let entity_builder = lazy_update
                    .create_entity(&entities)
                    .with(Model::new(block.mesh_id))
                    .with(Transform::from_position(
                        pos.x as f32,
                        pos.y as f32,
                        base.height,
                    ))
                    .with(GadgetEntity {
                        ship: ship_entity,
                        block_id: *block_id,
                    })
                    .with(Collider::new(
                        block.hitbox.clone(),
                        Collider::SHIP,
                        vec![Collider::ASTEROID],
                    ));
                let entity = if let Some(setup) = block.setup {
                    (setup)(entity_builder).build()
                } else {
                    entity_builder.build()
                };

                ship.tiles
                    .get_mut(pos)
                    .expect("Placed block outside ship boundries")
                    .gadget = Some(entity);
            }
            BuildAction::BuildBlock(pos, block_id) => {
                let block = blocks.get_block(*block_id);

                if block.size.x > 1 || block.size.y > 1 {
                    unimplemented!("Multiblock sizes not implemented!");
                }

                let entity_builder = lazy_update
                    .create_entity(&entities)
                    .with(Model::new(block.mesh_id))
                    .with(BlockEntity {
                        ship: ship_entity,
                        block_id: *block_id,
                        root: *pos,
                    })
                    .with(Transform::from_position(pos.x as f32, pos.y as f32, 0.0))
                    .with(Collider::new(
                        block.hitbox.clone(),
                        Collider::SHIP,
                        vec![Collider::ASTEROID],
                    ));
                let block_entity = if let Some(setup) = block.setup {
                    (setup)(entity_builder).build()
                } else {
                    entity_builder.build()
                };

                ship.tiles
                    .get_mut(pos)
                    .expect("Placed block outside ship boundries")
                    .block = Some(block_entity);
            }
            BuildAction::BuildFloor(pos, floor) => {
                let tile_entity = lazy_update
                    .create_entity(&entities)
                    .with(Model::new((*floor).into()))
                    .with(Transform::from_position(pos.x as f32, pos.y as f32, 0.0))
                    .build();

                ship.tiles
                    .get_mut(pos)
                    .expect("Placed floor outside ship boundries")
                    .block = Some(tile_entity);
            }
            _ => unimplemented!(),
        }
    }
}

pub fn create_ship(world: &mut World) -> Entity {
    let mut tiles = HashMap::new();
    let initial_size = 32;
    for x in -initial_size..initial_size {
        for y in -initial_size..initial_size {
            tiles.insert(
                (x, y).into(),
                Tile {
                    block: None,
                    gadget: None,
                    floor: None,
                },
            );
        }
    }

    let ship = world
        .create_entity()
        .with(Ship { tiles, heat: 0.0 })
        .build();
    let (ship_build_actions, ship_build_gadgets) = build_initial_ship(&world);

    execute_build_actions(world, ship, &ship_build_actions, true);
    // execute_build_actions adds the entities lazily, so we need to maintain the world
    // in order to add the block entities
    // TODO: This could cause a bug, because entities are not properly cleaned up with world.maintain()
    world.maintain();
    execute_build_actions(world, ship, &ship_build_gadgets, true);
    ship
}

fn build_initial_ship(world: &World) -> (Vec<BuildAction>, Vec<BuildAction>) {
    let blocks = world.fetch::<Blocks>();
    let metal_floor = world.fetch::<Floors>().metal;

    let mut ship = Vec::new();
    let mut gadgets = Vec::new();
    let size = 7;

    for x in 0..=size {
        for y in 0..=size {
            if x == 0 || y == 0 || x == size || y == size {
                ship.push(BuildAction::BuildBlock(Point2::new(x, y), blocks.wall));
            } else {
                ship.push(BuildAction::BuildFloor(Point2::new(x, y), metal_floor));
            }
        }
    }
    ship.push(BuildAction::BuildBlock(
        Point2::new(size + 1, -2),
        blocks.engine,
    ));
    ship.push(BuildAction::BuildBlock(
        Point2::new(size + 1, size + 2),
        blocks.engine,
    ));
    ship.push(BuildAction::BuildBlock(Point2::new(size, -2), blocks.cube));
    ship.push(BuildAction::BuildBlock(Point2::new(size, -1), blocks.cube));
    ship.push(BuildAction::BuildBlock(
        Point2::new(size, size + 1),
        blocks.cube,
    ));
    ship.push(BuildAction::BuildBlock(
        Point2::new(size, size + 2),
        blocks.cube,
    ));

    gadgets.push(BuildAction::BuildBlock(Point2::new(0, size), blocks.laser));
    gadgets.push(BuildAction::BuildBlock(
        Point2::new(1, size),
        blocks.cooler,
    ));
    gadgets.push(BuildAction::BuildBlock(Point2::new(1, 0), blocks.cooler));
    gadgets.push(BuildAction::BuildBlock(Point2::new(0, 0), blocks.laser));

    (ship, gadgets)
}
