use wasm_bindgen::prelude::*;

fn into_js_value<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&format!("{}", err))
}

#[wasm_bindgen]
pub enum KeyEvent {
    W,
    A,
    S,
    D,
    Space,
}

impl Into<winit::event::VirtualKeyCode> for KeyEvent {
    fn into(self) -> winit::event::VirtualKeyCode {
        use winit::event::VirtualKeyCode;
        match self {
            KeyEvent::W => VirtualKeyCode::W,
            KeyEvent::A => VirtualKeyCode::A,
            KeyEvent::S => VirtualKeyCode::S,
            KeyEvent::D => VirtualKeyCode::D,
            KeyEvent::Space => VirtualKeyCode::Space,
        }
    }
}

#[wasm_bindgen]
pub struct Wrapper {
    inner: crate::Game,
}

#[wasm_bindgen]
impl Wrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        debug_font_data: Vec<u8>,
    ) -> Result<Wrapper, JsValue> {
        let webgl_context = {
            use wasm_bindgen::JsCast;
            canvas
                .get_context("webgl")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::WebGlRenderingContext>()
                .unwrap()
        };
        let ctx = solstice_2d::solstice::glow::Context::from_webgl1_context(webgl_context);
        let ctx = solstice_2d::solstice::Context::new(ctx);

        let width = canvas.width();
        let height = canvas.height();

        let resources = crate::Resources { debug_font_data };

        let inner =
            crate::Game::new(ctx, width as _, height as _, resources).map_err(into_js_value)?;
        Ok(Self { inner })
    }

    #[wasm_bindgen]
    pub fn step(&mut self, _t_ms: f64) {
        // let t = duration_from_f64(t_ms);
        self.inner.update();
    }

    #[wasm_bindgen]
    pub fn handle_key_down(&mut self, key_code: KeyEvent) {
        let state = winit::event::ElementState::Pressed;
        self.inner.handle_key_event(state, key_code.into())
    }

    #[wasm_bindgen]
    pub fn handle_key_up(&mut self, key_code: KeyEvent) {
        let state = winit::event::ElementState::Released;
        self.inner.handle_key_event(state, key_code.into())
    }

    #[wasm_bindgen]
    pub fn handle_mouse_down(&mut self, is_left_button: bool) {
        let state = winit::event::ElementState::Pressed;
        let button = match is_left_button {
            true => winit::event::MouseButton::Left,
            false => winit::event::MouseButton::Right,
        };
        self.inner.handle_mouse_event(crate::MouseEvent::Button(state, button))
    }

    #[wasm_bindgen]
    pub fn handle_mouse_up(&mut self, is_left_button: bool) {
        let state = winit::event::ElementState::Released;
        let button = match is_left_button {
            true => winit::event::MouseButton::Left,
            false => winit::event::MouseButton::Right,
        };
        self.inner.handle_mouse_event(crate::MouseEvent::Button(state, button))
    }

    #[wasm_bindgen]
    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.inner.handle_mouse_event(crate::MouseEvent::Moved(x, y))
    }
}
