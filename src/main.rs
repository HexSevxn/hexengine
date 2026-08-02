use hexengine::engine::render::app::App;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app)
}

/*
todo

render loop fetches all entities with renderable, geometry, and transformation
on creation, entities with geometry, transform, and renderable get pushed into the triangle instance buffer to be drawn
wgsl shader must draw based on passed instance buffer, so need binding group to tell rpass.draw how to pass information
    (not a flat vertex walk for each triangle anymore)

switch the rendering over to an ECS compatible render loop, want high performance conversions from ECS types to renderable types (vertices)
move main loop over from condelve to try and get something more interesting rendered
maybe get input handling

create view buffer (memory allocation)
create bind group layout for view
create bindgroup for view

*/
