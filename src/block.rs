use crate::entity::{
    objects::{self, ObjectMeshes},
    Transform,
};
use crate::graphics::{self, MeshId, MeshManager};
use crate::InputAction;
use cgmath::{Point2, Vector3};
use specs::{prelude::*, world::LazyBuilder, Component};

pub type BlockId = usize;
pub type OnBlockSetup = fn(LazyBuilder) -> LazyBuilder;

pub struct Block {
    pub id: BlockId,
    pub type_name: &'static str,
    pub mesh_id: MeshId,
    /// The Size of the block in terms of grid spaces (x, y)
    pub size: Point2<u16>,
    /// The height of the block (z)
    pub height: f32,
    pub setup: Option<OnBlockSetup>,
}

pub struct Blocks {
    blocks: Vec<Block>,
    pub wall: BlockId,
    pub engine: BlockId,
    pub cube: BlockId,
    pub miner: BlockId,
}

impl Blocks {
    pub fn get_block(&self, id: BlockId) -> &Block {
        self.blocks
            .get(id)
            .unwrap_or_else(|| panic!("Invalid block ID:  {}", id))
    }
}

pub fn load_blocks(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> Blocks {
    let mut blocks = Vec::new();
    let wall = create_block(
        &mut blocks,
        mesh_manager.add(device, &graphics::load_mesh("wall")),
        (1, 1, 3.0),
        "wall",
        None,
    );
    let engine = create_block(
        &mut blocks,
        mesh_manager.add(device, &graphics::load_mesh("engine")),
        (1, 1, 1.0),
        "engine",
        None,
    );
    let cube = create_block(
        &mut blocks,
        mesh_manager.add(device, &graphics::load_mesh("box")),
        (1, 1, 1.0),
        "Box",
        None,
    );
    let miner = create_block(
        &mut blocks,
        mesh_manager.add(device, &graphics::load_mesh("miner")),
        (1, 1, 1.0),
        "Miner",
        Some(setup_miner),
    );

    Blocks {
        blocks,
        wall,
        engine,
        cube,
        miner,
    }
}

fn create_block(
    blocks: &mut Vec<Block>,
    mesh_id: MeshId,
    size: (u16, u16, f32),
    type_name: &'static str,
    setup: Option<OnBlockSetup>,
) -> BlockId {
    let id = blocks.len();
    let block = Block {
        id,
        mesh_id,
        type_name,
        setup,
        size: Point2::new(size.0, size.1),
        height: size.2,
    };

    println!("[Registered Block] {}={}", &block.type_name, id);
    blocks.push(block);
    id
}

pub fn register_components(world: &mut World) {
    world.register::<Miner>();
}

pub fn setup_systems(dispatcher: &mut DispatcherBuilder) {
    dispatcher.add(MinerSystem, "", &[]);
}

fn setup_miner(builder: LazyBuilder) -> LazyBuilder {
    println!("Building Miner!");
    builder.with(Miner::default())
}

#[derive(Component, Default)]
#[storage(HashMapStorage)]
pub struct Miner {
    shoot_time: u16,
}

impl Miner {
    const TOTAL_TIME: u16 = 120;
}

pub struct MinerSystem;

impl<'a> System<'a> for MinerSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadExpect<'a, InputAction>,
        ReadExpect<'a, ObjectMeshes>,
        WriteStorage<'a, Miner>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, lazy_update, input, meshes, mut miners, mut transforms) = data;

        for (transform, miner) in (&mut transforms, &mut miners).join() {
            transform.set_rotation_z(crate::PI);

            if miner.shoot_time > Miner::TOTAL_TIME {
                if let InputAction::Mining(target) = *input {
                    let position = transform.position + Vector3::new(0.0, 0.0, 0.5);
                    let builder = lazy_update.create_entity(&entities);
                    objects::build_mining_missle(&meshes, builder, target, position);
                    miner.shoot_time = 0;
                }
            } else {
                miner.shoot_time += 1;
            }
        }
    }
}
