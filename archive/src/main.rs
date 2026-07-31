use condelve::{
    engine::{
        ecs::{query::Query, world::World},
        math::Vec2,
    },
    client::ui::{Screen, Pixel},
    frame_loop,
    blocked_loop,
};

fn main() {
    let mut world = World::new();

    let mut screen = Screen::new();
    blocked_loop(&mut screen, &mut world);
}
