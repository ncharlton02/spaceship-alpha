use cgmath::{prelude::*, Point2, Point3, Vector3};
use graphics::{Camera, Mesh, Renderer, Vertex};
use winit::event;

pub const WIREFRAME_MODE: bool = false;

mod app;
mod graphics;

pub fn create_vertices() -> Vec<Vertex> {
    let red: Point3<f32> = (1.0, 0.0, 0.0).into();
    let blue: Point3<f32> = (0.0, 0.0, 1.0).into();
    let green: Point3<f32> = (0.0, 1.0, 0.0).into();
    let white: Point3<f32> = (1.0, 1.0, 1.0).into();
    let yellow: Point3<f32> = (1.0, 1.0, 0.0).into();
    let light_blue: Point3<f32> = (0.0, 1.0, 1.0).into();

    vec![
        // Bottom
        Vertex::new(-0.5, 0.5, 0.0, red),
        Vertex::new(-0.5, -0.5, 0.0, red),
        Vertex::new(0.5, -0.5, 0.0, red),
        Vertex::new(0.5, 0.5, 0.0, red),
        // Top
        Vertex::new(-0.5, 0.5, 1.0, blue),
        Vertex::new(-0.5, -0.5, 1.0, blue),
        Vertex::new(0.5, -0.5, 1.0, blue),
        Vertex::new(0.5, 0.5, 1.0, blue),
        // Left
        Vertex::new(-0.5, 0.5, 1.0, green),
        Vertex::new(-0.5, 0.5, 0.0, green),
        Vertex::new(-0.5, -0.5, 0.0, green),
        Vertex::new(-0.5, -0.5, 1.0, green),
        //Right
        Vertex::new(0.5, 0.5, 1.0, white),
        Vertex::new(0.5, 0.5, 0.0, white),
        Vertex::new(0.5, -0.5, 0.0, white),
        Vertex::new(0.5, -0.5, 1.0, white),
        //Front
        Vertex::new(0.5, 0.5, 1.0, yellow),
        Vertex::new(0.5, 0.5, 0.0, yellow),
        Vertex::new(-0.5, 0.5, 0.0, yellow),
        Vertex::new(-0.5, 0.5, 1.0, yellow),
        //Back
        Vertex::new(0.5, -0.5, 1.0, light_blue),
        Vertex::new(0.5, -0.5, 0.0, light_blue),
        Vertex::new(-0.5, -0.5, 0.0, light_blue),
        Vertex::new(-0.5, -0.5, 1.0, light_blue),
    ]
}

#[rustfmt::skip]
pub fn create_indices() -> Vec<u16>{
    vec![
        0, 1, 2, 0, 2, 3, //Bottom
        4, 5, 6, 4, 6, 7, //Top
        8, 9, 10, 8, 10, 11, //Left
        14, 13, 12, 15, 14, 12, //Right
        16, 17, 18, 16, 18, 19, //Front
        22, 21, 20, 23, 22, 20 //Back
    ]
}

struct AppState {
    renderer: Renderer,
    camera: Camera,
}

impl app::Application for AppState {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, _: &wgpu::Queue) -> Self {
        let mesh = Mesh {
            vertices: create_vertices(),
            indices: create_indices(),
        };
        let renderer = Renderer::new(device, &swapchain.format, &mesh);
        let camera = Camera {
            eye: (3.0, 3.0, 3.0).into(),
            target: Point3::origin(),
            up: Vector3::unit_z(),
            aspect: swapchain.width as f32 / swapchain.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        AppState { renderer, camera }
    }

    fn resize(&mut self, swapchain: &wgpu::SwapChainDescriptor, _: &wgpu::Device, _: &wgpu::Queue) {
        self.camera.aspect = swapchain.width as f32 / swapchain.height as f32;
    }

    fn key_event(&mut self, _: event::VirtualKeyCode, _: event::ElementState) {}

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, _: Point2<f32>) {}

    fn click_event(&mut self, _: event::MouseButton, _: event::ElementState, _: Point2<f32>) {}

    fn render(
        &mut self,
        texture: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let rotation = cgmath::Quaternion::from_angle_z(cgmath::Rad(0.02));

        self.camera.eye = rotation.rotate_point(self.camera.eye);
        self.renderer.render(device, queue, texture, &self.camera)
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}
