use crate::graphics::{Mesh, MeshId, MeshManager};
use cgmath::Point3;

const WALL_TYPE_NAME: &str = "wall";

pub type BlockId = usize;

pub struct Block {
    pub type_name: &'static str,
    pub mesh_id: MeshId,
    pub x_size: u16,
    pub y_size: u16,
}

pub struct Blocks {
    blocks: Vec<Block>,
    pub wall: BlockId,
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
    let wall = register_block(
        &mut blocks,
        Block {
            mesh_id: mesh_manager.add(
                device,
                &Mesh::rectangular_prism(1.0, 1.0, 4.0, Point3::new(0.8, 0.8, 0.8)),
            ),
            x_size: 1,
            y_size: 1,
            type_name: WALL_TYPE_NAME,
        },
    );

    Blocks { blocks, wall }
}

fn register_block(blocks: &mut Vec<Block>, block: Block) -> BlockId {
    let id = blocks.len();
    println!("[Registered Block] {}={}", &block.type_name, id);
    blocks.push(block);
    id
}
