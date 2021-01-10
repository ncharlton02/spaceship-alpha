use crate::graphics::{self, MeshId, MeshManager};
use cgmath::Point2;

pub type BlockId = usize;

pub struct Block {
    pub id: BlockId,
    pub type_name: &'static str,
    pub mesh_id: MeshId,
    /// The Size of the block in terms of grid spaces (x, y)
    pub size: Point2<u16>,
    /// The height of the block (z)
    pub height: f32,
}

pub struct Blocks {
    blocks: Vec<Block>,
    pub wall: BlockId,
    pub engine: BlockId,
    pub cube: BlockId,
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
    );
    let engine = create_block(
        &mut blocks,
        mesh_manager.add(device, &graphics::load_mesh("engine")),
        (1, 1, 1.0),
        "engine",
    );
    let cube = create_block(
        &mut blocks,
        mesh_manager.add(device, &graphics::load_mesh("box")),
        (1, 1, 1.0),
        "Box",
    );

    Blocks {
        blocks,
        wall,
        engine,
        cube,
    }
}

fn create_block(
    blocks: &mut Vec<Block>,
    mesh_id: MeshId,
    size: (u16, u16, f32),
    type_name: &'static str,
) -> BlockId {
    let id = blocks.len();
    let block = Block {
        id,
        mesh_id,
        type_name,
        size: Point2::new(size.0, size.1),
        height: size.2,
    };

    println!("[Registered Block] {}={}", &block.type_name, id);
    blocks.push(block);
    id
}
