use super::{Collider, ColliderShape, Model, Transform};
use crate::block::{BlockId, Blocks};
use crate::floor::{Floor, Floors};
use cgmath::{Point2, Vector3};
use specs::{prelude::*, world::EntitiesRes, Component};
use std::collections::HashMap;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Ship {
    tiles: HashMap<Point2<i16>, Tile>,
}

#[derive(Clone)]
pub struct Tile {
    block: Option<Entity>,
    floor: Option<Entity>,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct BlockEntity {
    root: Point2<i16>,
}

pub enum BuildAction {
    BuildBlock(Point2<i16>, BlockId),
    RemoveBlock(Point2<i16>),
    BuildFloor(Point2<i16>, Floor),
    RemoveFloor(Point2<i16>),
}

pub fn execute_build_actions(world: &mut World, ship: Entity, actions: &[BuildAction]) {
    let lazy_update = world.fetch::<LazyUpdate>();
    let entities = world.fetch::<EntitiesRes>();
    let mut ships = world.write_component::<Ship>();
    let ship = ships.get_mut(ship).unwrap();

    for action in actions {
        match action {
            BuildAction::BuildBlock(pos, block_id) => {
                let blocks = world.fetch::<Blocks>();
                let block = blocks.get_block(*block_id);

                if block.size.x > 1 || block.size.y > 1 {
                    unimplemented!("Multiblock sizes not implemented!");
                }

                let entity_builder = lazy_update
                    .create_entity(&entities)
                    .with(Model::new(block.mesh_id))
                    .with(BlockEntity { root: *pos })
                    .with(Transform::from_position(
                        pos.x as f32,
                        pos.y as f32,
                        block.height / 2.0,
                    ))
                    .with(Collider {
                        shape: ColliderShape::Cuboid(Vector3::new(
                            block.size.x as f32,
                            block.size.y as f32,
                            block.height,
                        )),
                        group: Collider::SHIP,
                        whitelist: vec![Collider::ASTEROID],
                    });
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

pub fn create_ship(world: &mut World) {
    let mut tiles = HashMap::new();
    let initial_size = 32;
    for x in -initial_size..initial_size {
        for y in -initial_size..initial_size {
            tiles.insert(
                (x, y).into(),
                Tile {
                    block: None,
                    floor: None,
                },
            );
        }
    }

    let ship = world.create_entity().with(Ship { tiles }).build();
    let build_actions = build_initial_ship(&world);

    execute_build_actions(world, ship, &build_actions);
}

fn build_initial_ship(world: &World) -> Vec<BuildAction> {
    let blocks = world.fetch::<Blocks>();

    let floor = world.fetch::<Floors>().metal;
    let mut actions = Vec::new();
    let size = 7;

    for x in 0..=size {
        for y in 0..=size {
            if x == 0 || y == 0 || x == size || y == size {
                actions.push(BuildAction::BuildBlock(Point2::new(x, y), blocks.wall));
            } else {
                actions.push(BuildAction::BuildFloor(Point2::new(x, y), floor));
            }
        }
    }
    actions.push(BuildAction::BuildBlock(
        Point2::new(size + 1, -2),
        blocks.engine,
    ));
    actions.push(BuildAction::BuildBlock(Point2::new(size, -2), blocks.cube));
    actions.push(BuildAction::BuildBlock(Point2::new(size, -1), blocks.cube));
    actions.push(BuildAction::BuildBlock(
        Point2::new(size, size + 1),
        blocks.cube,
    ));
    actions.push(BuildAction::BuildBlock(
        Point2::new(size, size + 2),
        blocks.cube,
    ));
    actions.push(BuildAction::BuildBlock(
        Point2::new(-1, size / 2),
        blocks.miner,
    ));
    actions.push(BuildAction::BuildBlock(
        Point2::new(size + 1, size + 2),
        blocks.engine,
    ));

    actions
}
