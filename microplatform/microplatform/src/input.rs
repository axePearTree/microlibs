use crate::Key;

#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
}

#[derive(Clone, Debug, Default)]
pub struct KeyboardState {
    keys: [KeyState; Key::Count as usize],
}

// TODO: instead of 3 bools let's use a single byte with bitflags and reduce the size of this struct
#[derive(Copy, Clone, Debug, Default)]
struct KeyState {
    down: bool,
    just_down: bool,
    just_up: bool,
}

impl KeyboardState {
    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys[key as usize].down
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        !self.keys[key as usize].down
    }

    pub fn is_key_just_down(&self, key: Key) -> bool {
        !self.keys[key as usize].just_down
    }

    pub fn is_key_just_up(&self, key: Key) -> bool {
        !self.keys[key as usize].just_up
    }

    pub(crate) fn clear_memory(&mut self) {
        for key in self.keys.iter_mut() {
            key.just_down = false;
            key.just_up = false;
        }
    }

    pub(crate) fn on_key_down(&mut self, key: Key) {
        self.keys[key as usize].down = true;
        self.keys[key as usize].just_down = true;
        self.keys[key as usize].just_up = false;
    }

    pub(crate) fn on_key_up(&mut self, key: Key) {
        self.keys[key as usize].down = false;
        self.keys[key as usize].just_down = false;
        self.keys[key as usize].just_up = false;
    }
}

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    position: (i32, i32),
    pub left: MouseButtonState,
    pub right: MouseButtonState,
}

#[derive(Clone, Debug, Default)]
pub struct MouseButtonState {
    down: bool,
    just_down: bool,
    just_up: bool,
    double_click: bool,
}

impl MouseState {
    #[inline]
    pub fn position(&self) -> (i32, i32) {
        self.position
    }

    pub(crate) fn set_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
    }

    pub(crate) fn clear_memory(&mut self) {
        self.left.clear_memory();
        self.right.clear_memory();
    }
}

impl MouseButtonState {
    #[inline]
    pub fn is_down(&self) -> bool {
        self.down
    }

    #[inline]
    pub fn is_up(&self) -> bool {
        !self.down
    }

    #[inline]
    pub fn is_just_down(&self) -> bool {
        self.just_down
    }

    #[inline]
    pub fn is_just_up(&self) -> bool {
        self.just_up
    }

    #[inline]
    pub fn is_double_click(&self) -> bool {
        self.double_click
    }

    pub(crate) fn on_down(&mut self) {
        self.down = true;
        self.just_down = true;
        self.just_up = false;
    }

    pub(crate) fn on_up(&mut self) {
        self.down = false;
        self.just_down = false;
        self.just_up = true;
    }

    pub(crate) fn on_double_click(&mut self) {
        self.down = false;
        self.just_down = true;
        self.just_up = false;
        self.double_click = true;
    }

    fn clear_memory(&mut self) {
        self.just_down = false;
        self.just_up = false;
        self.double_click = false;
    }
}

