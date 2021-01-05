use super::{Model, SimpleStorage, Transform};
use crate::block::{Block, BlockId, Blocks};
use crate::floor::{Floor, Floors};
use cgmath::Point2;
use specs::{prelude::*, Component};
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

/// A system that will build a block at a point
pub struct ShipBuildSystem {
    ship: Entity,
    actions: Vec<BuildAction>,
}

impl ShipBuildSystem {
    fn build_block(
        pos: Point2<i16>,
        tiles: &mut HashMap<Point2<i16>, Tile>,
        block: &Block,
        entities: &Entities,
        models: &mut SimpleStorage<'_, Model>,
        block_entities: &mut SimpleStorage<'_, BlockEntity>,
        transforms: &mut SimpleStorage<'_, Transform>,
    ) {
        if block.size.x > 1 || block.size.y > 1 {
            unimplemented!();
        }

        let tile = tiles
            .get_mut(&pos)
            .unwrap_or_else(|| panic!("Invalid Tile: {:?}", pos));
        let block_entity = entities.create();

        tile.block = Some(block_entity);
        models
            .insert(block_entity, Model::new(block.mesh_id))
            .unwrap();
        block_entities
            .insert(block_entity, BlockEntity { root: pos })
            .unwrap();
        transforms
            .insert(
                block_entity,
                Transform::from_position(pos.x as f32, pos.y as f32, 0.0),
            )
            .unwrap();

        // println!("Built {} ({}, {})", block.type_name, pos.x, pos.y);
    }

    fn build_floor(
        pos: Point2<i16>,
        tiles: &mut HashMap<Point2<i16>, Tile>,
        floor: Floor,
        entities: &Entities,
        models: &mut SimpleStorage<'_, Model>,
        transforms: &mut SimpleStorage<'_, Transform>,
    ) {
        let tile = tiles
            .get_mut(&pos)
            .unwrap_or_else(|| panic!("Invalid Tile: {:?}", pos));
        let tile_entity = entities.create();

        tile.floor = Some(tile_entity);
        models
            .insert(tile_entity, Model::new(floor.into()))
            .unwrap();
        transforms
            .insert(
                tile_entity,
                Transform::from_position(pos.x as f32, pos.y as f32, 0.0),
            )
            .unwrap();

        // println!("Built Floor ({}, {})", pos.x, pos.y);
    }
}

impl<'a> System<'a> for ShipBuildSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Blocks>,
        WriteStorage<'a, Ship>,
        WriteStorage<'a, BlockEntity>,
        WriteStorage<'a, Model>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, blocks, mut ships, mut block_entities, mut models, mut transforms) = data;
        let tiles = &mut ships.get_mut(self.ship).unwrap().tiles;

        for action in &self.actions {
            match action {
                BuildAction::BuildBlock(pos, block_id) => {
                    let block = blocks.get_block(*block_id);
                    ShipBuildSystem::build_block(
                        *pos,
                        tiles,
                        block,
                        &entities,
                        &mut models,
                        &mut block_entities,
                        &mut transforms,
                    );
                }
                BuildAction::BuildFloor(pos, floor) => ShipBuildSystem::build_floor(
                    *pos,
                    tiles,
                    *floor,
                    &entities,
                    &mut models,
                    &mut transforms,
                ),
                _ => unimplemented!(),
            }
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
    build_initial_ship(world, ship);
}

fn build_initial_ship(world: &mut World, ship: Entity) {
    let wall = world.fetch::<Blocks>().wall;
    let engine = world.fetch::<Blocks>().engine;
    let cube = world.fetch::<Blocks>().cube;

    let floor = world.fetch::<Floors>().metal;
    let mut actions = Vec::new();
    let size = 7;

    for x in 0..=size {
        for y in 0..=size {
            if x == 0 || y == 0 || x == size || y == size {
                actions.push(BuildAction::BuildBlock(Point2::new(x, y), wall));
            } else {
                actions.push(BuildAction::BuildFloor(Point2::new(x, y), floor));
            }
        }
    }
    actions.push(BuildAction::BuildBlock(Point2::new(size + 1, -2), engine));
    actions.push(BuildAction::BuildBlock(Point2::new(size, -2), cube));
    actions.push(BuildAction::BuildBlock(Point2::new(size, -1), cube));
    actions.push(BuildAction::BuildBlock(Point2::new(size, size + 1), cube));
    actions.push(BuildAction::BuildBlock(Point2::new(size, size + 2), cube));
    actions.push(BuildAction::BuildBlock(
        Point2::new(size + 1, size + 2),
        engine,
    ));

    ShipBuildSystem { ship, actions }.run_now(&world);
}
