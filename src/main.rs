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

camera view through uniform shader and "global" entity
    -- requires its own buffer

do something with inputs
text rendering
*/
