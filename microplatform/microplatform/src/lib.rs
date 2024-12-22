#[macro_use]
extern crate alloc;

pub mod backend;
pub mod canvas;
pub mod font;
pub mod input;
mod text;
pub mod texture;
pub mod types;

use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec::Vec;
use backend::*;
use canvas::Canvas;
use core::cell::RefCell;
use font::Font;
use input::InputState;
use texture::*;
use types::*;

pub type Result<T = ()> = core::result::Result<T, String>;
pub(crate) type BackendRef = Rc<RefCell<dyn Backend>>;
pub(crate) type BackendWeakRef = Weak<RefCell<dyn Backend>>;

pub trait Application {
    fn update(&mut self, context: &mut Context, delta_ms: u64) -> Result;
    fn fixed_update(&mut self, context: &mut Context, fixed_ms: u64) -> Result;
    fn draw(&mut self, canvas: &mut Canvas, alpha_secs: f32) -> Result;
}

pub struct Context {
    pub(crate) backend: BackendRef,
    input: InputState,
    events: Vec<Event>,
    quit: bool,
}

impl Context {
    pub fn new(context: impl Backend + 'static) -> Self {
        Self {
            backend: Rc::new(RefCell::new(context)),
            events: Vec::with_capacity(16),
            input: InputState::default(),
            quit: false,
        }
    }

    pub fn set_window_config(&mut self, config: WindowConfig) -> Result {
        self.backend.borrow_mut().window_set_config(config)
    }

    pub fn load_texture(&mut self, path: &str) -> Result<Texture> {
        Texture::new_static(&self.backend, path)
    }

    pub fn create_target(&mut self, w: u32, h: u32) -> Result<Texture> {
        Texture::new_target(&self.backend, w, h)
    }

    pub fn load_font(&mut self, path: &str, scale: u8) -> Result<Font> {
        Font::new(&self.backend, path, scale)
    }

    pub fn request_quit(&mut self) {
        self.quit = true;
    }

    pub fn input(&self) -> InputState {
        self.input.clone()
    }

    fn refresh_events(&mut self) {
        self.events.clear();
        self.backend.borrow_mut().events_pump(&mut self.events);
    }

    fn update_mouse_position(&mut self) -> Result {
        let pos = self.backend.borrow_mut().input_mouse_position()?;
        self.input.mouse.set_position(pos.0, pos.1);
        Ok(())
    }

    fn canvas(&self) -> Result<Canvas> {
        self.backend.borrow_mut().render_clear()?;
        Canvas::new(&self.backend, None)
    }

    fn millis(&self) -> Result<u64> {
        self.backend.borrow_mut().system_get_millis()
    }
}

pub fn run_event_loop<T: Application>(
    backend: impl Backend + 'static,
    load: impl FnOnce(&mut Context) -> Result<T>,
) -> Result {
    const FIXED_TIMESTEP_MILLIS: u64 = 16;

    let mut context = Context::new(backend);

    let mut app = load(&mut context)?;

    let mut millis_now = context.millis()?;
    let mut acc_millis = 0;

    'game_loop: loop {
        let millis_before = millis_now;
        millis_now = context.millis()?;

        let delta_millis = millis_now - millis_before;
        acc_millis += delta_millis;

        context.update_mouse_position()?;
        context.input.keyboard.clear_memory();
        context.input.mouse.clear_memory();
        context.refresh_events();
        for event in context.events.iter() {
            #[allow(unreachable_patterns)]
            match event {
                Event::KeyDown(key) => context.input.keyboard.on_key_down(*key),
                Event::KeyUp(key) => context.input.keyboard.on_key_up(*key),
                Event::MouseLeftButtonDown => context.input.mouse.left.on_down(),
                Event::MouseLeftButtonUp => context.input.mouse.left.on_up(),
                Event::MouseLeftButtonDoubleClick => context.input.mouse.left.on_double_click(),
                Event::MouseRightButtonDown => context.input.mouse.right.on_down(),
                Event::MouseRightButtonUp => context.input.mouse.right.on_up(),
                Event::MouseRightButtonDoubleClick => context.input.mouse.right.on_double_click(),
                Event::Close => break 'game_loop,
                _ => {}
            }
        }

        app.update(&mut context, delta_millis)?;

        if acc_millis >= FIXED_TIMESTEP_MILLIS {
            acc_millis -= FIXED_TIMESTEP_MILLIS;
            app.fixed_update(&mut context, FIXED_TIMESTEP_MILLIS)?;
        }

        let alpha = acc_millis as f32 / FIXED_TIMESTEP_MILLIS as f32;

        app.draw(&mut context.canvas()?, alpha)?;

        if context.quit {
            break 'game_loop;
        }
    }

    Ok(())
}
