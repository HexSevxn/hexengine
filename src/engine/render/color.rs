use glam::{Vec3, Vec4, vec3, vec4};

#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TriangleColorMap {
    pub c0: Color,
    pub c1: Color,
    pub c2: Color,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn as_vec4(&self) -> Vec4 {
        return vec4(self.r, self.g, self.b, self.a);
    }

    pub fn from_vec4(color: Vec4) -> Color {
        return Color {
            r: color.x,
            g: color.y,
            b: color.z,
            a: color.w,
        };
    }

    pub fn as_vec3(&self) -> Vec3 {
        return vec3(self.r, self.g, self.b);
    }

    pub fn from_vec3(color: Vec3) -> Color {
        return Color {
            r: color.x,
            g: color.y,
            b: color.z,
            a: 1.0,
        };
    }

    pub fn as_rgb(&self) -> [f32; 3] {
        return [self.r, self.g, self.b];
    }
    pub fn as_rgba(&self) -> [f32; 4] {
        return [self.r, self.g, self.b, self.a];
    }
}

impl TriangleColorMap {
    pub fn convert_raw(self) -> ([f32; 4], [f32; 4], [f32; 4]) {
        return self.into();
    }

    pub fn flat(color: Vec4) -> TriangleColorMap {
        let flat_color = Color::from_vec4(color);
        return TriangleColorMap {
            c0: flat_color,
            c1: flat_color,
            c2: flat_color,
        };
    }
}

impl Into<([f32; 4], [f32; 4], [f32; 4])> for TriangleColorMap {
    fn into(self) -> ([f32; 4], [f32; 4], [f32; 4]) {
        return (self.c0.as_rgba(), self.c1.as_rgba(), self.c2.as_rgba());
    }
}
