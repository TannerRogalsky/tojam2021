#[cfg(not(target_arch = "wasm32"))]
mod native {
    use glutin as winit;
    use solstice_2d::solstice::glow::Context;
    use winit::{
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
    };

    type WindowContext = winit::ContextWrapper<winit::PossiblyCurrent, winit::window::Window>;

    pub struct NativeWindow {
        inner: WindowContext,
    }

    impl NativeWindow {
        pub fn new(inner: WindowContext) -> Self {
            Self { inner }
        }

        pub fn swap_buffers(&self) -> eyre::Result<()> {
            self.inner.swap_buffers().map_err(eyre::Report::new)
        }
    }

    impl std::ops::Deref for NativeWindow {
        type Target = Window;

        fn deref(&self) -> &Self::Target {
            &self.inner.window()
        }
    }

    pub fn init_ctx(wb: WindowBuilder, el: &EventLoop<()>) -> (Context, NativeWindow) {
        let windowed_context = winit::ContextBuilder::new()
            .with_multisampling(16)
            .with_vsync(true)
            .build_windowed(wb, &el)
            .unwrap();
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };
        let gfx = unsafe {
            Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _)
        };
        (gfx, NativeWindow::new(windowed_context))
    }
}

#[cfg(target_arch = "wasm32")]
mod websys {
    use solstice_2d::solstice::glow::Context;
    use winit::{
        event_loop::EventLoop,
        platform::web::*,
        window::{Window, WindowBuilder},
    };

    pub struct WebsysWindow {
        inner: Window,
    }

    impl WebsysWindow {
        pub fn new(inner: Window) -> Self {
            Self { inner }
        }

        pub fn canvas(&self) -> web_sys::HtmlCanvasElement {
            self.inner.canvas()
        }

        pub fn swap_buffers(&self) -> eyre::Result<()> {
            Ok(())
        }
    }

    impl std::ops::Deref for WebsysWindow {
        type Target = Window;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    pub fn init_ctx(wb: WindowBuilder, el: &EventLoop<()>) -> (Context, WebsysWindow) {
        use wasm_bindgen::JsCast;

        let canvas = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|doc| doc.get_element_by_id("canvas"))
            .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok());
        let window = wb.with_canvas(canvas).build(&el).unwrap();
        let webgl_context = window
            .canvas()
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGlRenderingContext>()
            .unwrap();
        let gfx = Context::from_webgl1_context(webgl_context);
        (gfx, WebsysWindow::new(window))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use {
    glutin as winit,
    native::{init_ctx, NativeWindow as Window},
};
#[cfg(target_arch = "wasm32")]
pub use {
    websys::{init_ctx, WebsysWindow as Window},
    winit,
};
