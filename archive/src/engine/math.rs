#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rec {
    pub aa: Vec2,
    pub bb: Vec2,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Circle {
    pub center: Vec2,
    pub radius: i32,
}

pub fn vec2(x: i32, y: i32) -> Vec2 {
    return Vec2 {x, y};
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Vec2 {
        return Vec2 { x, y };
    }
    pub fn length(&self) -> f32 {
        return ((self.x.pow(2) + self.y.pow(2)) as f32).sqrt();
    }
    pub fn normal(&self) -> Vec2 {
        return Vec2 {x: self.x.signum(), y: self.y.signum()};
    }

    pub fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
        return (a + b) / 2;
    }

    pub const fn zero() -> Vec2 {
        return Vec2 {x: 0, y: 0};
    }
}

impl Rec {
    pub fn new(a: Vec2, b: Vec2) -> Rec {
        return Rec {aa: a, bb: b};
    }

    pub const fn ab(&self) -> Vec2 {
        return vec(self.aa.x, self.bb.y);
    }

    pub const fn ba(&self) -> Vec2 {
        return vec(self.bb.x, self.aa.y);
    }

    pub fn center(&self) -> Vec2 {
        return Vec2::midpoint(self.aa, self.bb);
    }

    pub fn perimeter(&self) -> i32 {
        let distance = self.bb - self.aa;
        return (distance.x * 2) + (distance.y * 2);
    }

    pub fn quarter(&self) -> [Self; 4] {
        let center = self.center();
        let distance = center - self.aa;
        let dist_x = vec2(distance.x, 0);
        let dist_y = vec2(0, distance.y);

        [
            Rect::new(self.aa, center),
            Rect::new(self.aa + dist_x, center + dist_x),
            Rect::new(self.aa + dist_y, center + dist_y),
            Rect::new(center, self.bb),
        ]
    }
}

impl Circle {
    fn new(center: Vec2, radius: i32) -> Circle {
        return Circle {center, radius};
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Sub for &Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Self) -> Self::Output {
        return Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        return Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::cmp::PartialEq for &Vec2 {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }
    fn ne(&self, other: &Self) -> bool {
        return self.x != other.x && self.y != other.y;
    }
}

impl From<(u16, u16)> for Vec2 {
    fn from(value: (u16, u16)) -> Self {
        return Vec2::new(value.0 as i32, value.1 as i32);
    }
}

impl From<Vec2> for (u16, u16) {
    fn from(value: Vec2) -> Self {
        return (value.x as u16, value.y as u16);
    }
}