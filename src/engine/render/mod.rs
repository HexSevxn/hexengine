pub mod wgpu_ctx;
pub mod app;
pub mod color;

use glam::{Vec2, Vec4};
use wgpu;

use crate::engine::asset::Asset;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    pub v1: Vertex,
    pub v2: Vertex,
    pub v3: Vertex,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn from_vec2_c(vec: Vec2, color: Vec4) -> Vertex {
        Vertex {
            position: [vec.x, vec.y],
            color: color.into(),
        }
    }
}

impl From<Vec2> for Vertex {
    fn from(value: Vec2) -> Self {
        return Vertex {
            position: value.into(),
            color: [1.0, 1.0, 1.0, 1.0],
        };
    }
}

impl Triangle {
    pub fn new(v1: Vertex, v2: Vertex, v3: Vertex) -> Triangle {
        return Triangle { v1, v2, v3 };
    }
}

impl Asset for Triangle {
    fn unpack(&self) -> Vec<Triangle> {
        return vec![self.clone()];
    }
}
