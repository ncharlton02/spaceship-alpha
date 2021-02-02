use crate::entity::{
    objects::{self, Health, ObjectMeshes},
    ColliderShape, Hitbox, Line, RaycastWorld, Transform,
};
use crate::graphics::{self, Mesh, MeshId, MeshManager};
use crate::InputAction;
use cgmath::{prelude::*, Point2, Vector3};
use specs::{prelude::*, world::LazyBuilder, Component};

pub type BlockId = usize;
pub type OnBlockSetup = fn(LazyBuilder) -> LazyBuilder;

// TODO: Currently size is used for collision and grid spaces (but they should seperate)
pub struct Block {
    pub id: BlockId,
    pub type_name: &'static str,
    pub mesh_id: MeshId,
    /// The Size of the block in terms of grid spaces (x, y)
    pub size: Point2<u16>,
    /// The height of the block (z)
    pub height: f32,
    pub hitbox: Hitbox,
    pub setup: Option<OnBlockSetup>,
    pub is_gadget: bool,
}

pub struct Blocks {
    blocks: Vec<Block>,
    pub wall: BlockId,
    pub engine: BlockId,
    pub cube: BlockId,
    pub miner: BlockId,
    pub laser: BlockId,
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
    let mut register_mesh = |mesh: &Mesh| {
        let id = mesh_manager.add(device, mesh);
        mesh_manager.set_mesh_visisble(id, crate::RENDER_BLOCKS);
        id
    };

    let wall = create_block(
        &mut blocks,
        register_mesh(&graphics::load_mesh("wall")),
        (1, 1, 3.0),
        None,
        "wall",
        None,
        false,
    );
    let engine = create_block(
        &mut blocks,
        register_mesh(&graphics::load_mesh("engine")),
        (1, 1, 1.0),
        None,
        "engine",
        None,
        false,
    );
    let cube = create_block(
        &mut blocks,
        register_mesh(&graphics::load_mesh("box")),
        (1, 1, 1.0),
        None,
        "Box",
        None,
        false,
    );
    let miner = create_block(
        &mut blocks,
        register_mesh(&graphics::load_mesh("miner")),
        (1, 1, 1.0),
        None,
        "Miner",
        Some(setup_miner),
        false,
    );
    let laser = create_block(
        &mut blocks,
        register_mesh(&graphics::load_mesh("laser")),
        (1, 1, 0.2),
        Some(Hitbox::new(
            ColliderShape::Cuboid(Vector3::new(0.6, 0.6, 0.525)),
            Vector3::new(0.0, 0.0, 0.525 / 2.0),
        )),
        "Laser",
        Some(setup_laser),
        true,
    );

    Blocks {
        blocks,
        wall,
        engine,
        cube,
        miner,
        laser,
    }
}

fn create_block(
    blocks: &mut Vec<Block>,
    mesh_id: MeshId,
    size: (u16, u16, f32),
    hitbox: Option<Hitbox>,
    type_name: &'static str,
    setup: Option<OnBlockSetup>,
    is_gadget: bool,
) -> BlockId {
    let id = blocks.len();
    let block = Block {
        id,
        mesh_id,
        type_name,
        setup,
        is_gadget,
        hitbox: hitbox.unwrap_or(Hitbox::new(
            ColliderShape::Cuboid(Vector3::new(size.0 as f32, size.1 as f32, size.2)),
            Vector3::new(0.0, 0.0, size.2 / 2.0),
        )),
        size: Point2::new(size.0, size.1),
        height: size.2,
    };

    println!("[Registered Block] {}={}", &block.type_name, id);
    blocks.push(block);
    id
}

pub fn register_components(world: &mut World) {
    world.register::<Miner>();
    world.register::<Laser>();
}

pub fn setup_systems(dispatcher: &mut DispatcherBuilder) {
    dispatcher.add(MinerSystem, "", &[]);
    dispatcher.add(LaserSystem, "", &[]);
}

fn setup_miner(builder: LazyBuilder) -> LazyBuilder {
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

fn setup_laser(builder: LazyBuilder) -> LazyBuilder {
    builder.with(Laser)
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Laser;

pub struct LaserSystem;

impl<'a> System<'a> for LaserSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, InputAction>,
        ReadExpect<'a, RaycastWorld>,
        WriteStorage<'a, Laser>,
        WriteStorage<'a, Line>,
        WriteStorage<'a, Health>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, input, raycaster, lasers, mut lines, mut healths, mut transforms) = data;

        for (entity, _) in (&entities, &lasers).join() {
            if let InputAction::Laser(target) = *input {
                let target_pos = transforms.get(target).unwrap().position;
                let transform = transforms.get_mut(entity).unwrap();
                let mut start_pos = transform.position + Vector3::new(0.0, 0.0, 0.4);
                let angle_xy = (start_pos.y - target_pos.y).atan2(start_pos.x - target_pos.x);
                let radius = 0.35;
                start_pos -= radius * Vector3::new(angle_xy.cos(), angle_xy.sin(), 0.0);

                let raycast = raycaster.raycast(Vec::with_capacity(0), start_pos, target_pos);

                if Some(target) == raycast {
                    transform.set_rotation_z(angle_xy);

                    lines
                        .insert(
                            entity,
                            Line {
                                pt: start_pos,
                                pt2: target_pos,
                                color: Vector3::new(1.0, 0.0, 0.0),
                            },
                        )
                        .expect("Unable to set line component for laser!");

                    if let Some(health) = healths.get_mut(target) {
                        health.damage(1);
                    }

                    continue;
                }
            }

            lines.remove(entity);
        }
    }
}
