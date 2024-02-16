use std::collections::BTreeMap;

pub type Key = winit::keyboard::KeyCode;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum KeyState {
    Pressed,
    Released,
}

pub struct State {
    keys: BTreeMap<Key, KeyState>,
}

impl State {
    pub fn get_key_state(&self, key: Key) -> KeyState {
        if let Some(state) = self.keys.get(&key) {
            *state
        } else {
            KeyState::Released
        }
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.get_key_state(key) == KeyState::Pressed
    }

    pub fn is_key_released(&self, key: Key) -> bool {
        self.get_key_state(key) == KeyState::Released
    }
}

pub struct Input {
    state: State,
}

impl Input {
    pub fn new() -> Self {
        Self {
            state: State {
                keys: BTreeMap::new(),
            },
        }
    }

    pub fn on_key_state_change(&mut self, key: Key, new_state: KeyState) {
        if let Some(key_state) = self.state.keys.get_mut(&key) {
            *key_state = new_state;
        } else {
            self.state.keys.insert(key, new_state);
        }
    }

    pub fn get_state<'a>(&'a self) -> &'a State {
        &self.state
    }
}