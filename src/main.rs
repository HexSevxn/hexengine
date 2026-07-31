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

switch the rendering over to an ECS compatible render loop, want high performance conversions from ECS types to renderable types (vertices)
move main loop over from condelve to try and get something more interesting rendered
maybe get input handling

implement a "view" uniform buffer for the screen size/ratio, to allow for aspect ratio scaling to prevent the terrible stretching
create view buffer (memory allocation)
create bind group layout for view
create bindgroup for view

*/
