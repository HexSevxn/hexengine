pub mod app;
pub mod color;
pub mod wgpu_ctx;

use crate::engine::render::color::TriangleColorMap;
use glam::Vec2;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    pub position: [f32; 2],
    pub rotation: f32,
    pub _pad: f32,

    pub v0: [f32; 2],
    pub v1: [f32; 2],
    pub v2: [f32; 2],
    pub _pad2: [f32; 2],

    pub c0: [f32; 4],
    pub c1: [f32; 4],
    pub c2: [f32; 4],
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            position: [0.0, 0.0],
            rotation: 0.0,
            _pad: 0.0,

            v0: [0.0, 0.0],
            v1: [1.0, 0.0],
            v2: [0.0, 1.0],
            _pad2: [0.0, 0.0],

            c0: [1.0, 1.0, 1.0, 1.0],
            c1: [1.0, 1.0, 1.0, 1.0],
            c2: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl Triangle {
    pub fn new(v0: Vec2, v1: Vec2, v2: Vec2, color_map: TriangleColorMap) -> Triangle {
        let (c0, c1, c2) = color_map.convert_raw();
        return Triangle {
            v0: v0.into(),
            v1: v1.into(),
            v2: v2.into(),
            c0,
            c1,
            c2,
            ..Default::default()
        };
    }
}
