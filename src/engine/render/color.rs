use glam::{Vec4, vec4};

#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub const WHITE: Color = Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0};
    pub const BLACK: Color = Color {r: 0.0, g: 0.0, b: 0.0, a: 0.0};
    pub const RED: Color = Color {r: 1.0, g: 0.0, b: 0.0, a: 1.0};
    pub const GREEN: Color = Color {r: 0.0, g: 1.0, b: 0.0, a: 1.0};
    pub const BLUE: Color = Color {r: 0.0, g: 0.0, b: 1.0, a: 1.0};

    pub fn as_vec4(&self) -> Vec4 {
        return vec4(self.r, self.g, self.b, self.a)
    }
}