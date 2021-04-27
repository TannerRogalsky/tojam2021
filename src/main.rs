use tojam2021::*;

mod window;

fn main() -> eyre::Result<()> {
    let (width, height) = (1280, 720);
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("tojam")
        .with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
    let (glow_ctx, window) = window::init_ctx(wb, &event_loop);
    let ctx = solstice_2d::solstice::Context::new(glow_ctx);

    let resources_folder = std::path::PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("docs");
    let resources = Resources {
        debug_font_data: std::fs::read(resources_folder.join("Inconsolata-Regular.ttf"))?,
    };

    let mut game = Game::new(ctx, width as _, height as _, resources)?;

    event_loop.run(move |event, _, cf| {
        use glutin::{event::*, event_loop::ControlFlow};
        match event {
            Event::NewEvents(_) => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    game.handle_resize(size.width as _, size.height as _);
                }
                WindowEvent::CloseRequested => {
                    *cf = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key_code),
                            ..
                        },
                    ..
                } => game.handle_key_event(state, key_code),
                WindowEvent::MouseInput { state, button, .. } => {
                    game.handle_mouse_event(MouseEvent::Button(state, button));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    game.handle_mouse_event(MouseEvent::Moved(position.x as _, position.y as _));
                }
                _ => {}
            },
            Event::DeviceEvent { .. } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::Resumed => {}
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                game.update();
                window.swap_buffers().expect("omfg");
            }
            Event::RedrawEventsCleared => {}
            Event::LoopDestroyed => {}
        }
    });
}
