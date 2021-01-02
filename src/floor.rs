use crate::graphics::{Mesh, MeshId, MeshManager};
use cgmath::Point3;

#[derive(Clone, Copy)]
pub struct Floor(MeshId);

impl Into<MeshId> for Floor {
    fn into(self) -> MeshId {
        self.0
    }
}

pub struct Floors {
    pub metal: Floor,
}

pub fn load_floors(device: &wgpu::Device, mesh_manager: &mut MeshManager) -> Floors {
    Floors {
        metal: Floor(mesh_manager.add(
            device,
            &Mesh::rectangular_prism(1.0, 1.0, 0.1, Point3::new(0.9, 0.9, 1.0)),
        )),
    }
}
