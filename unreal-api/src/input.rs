use unreal_ffi::ActionState;

use crate::module::bindings;
use std::{collections::HashMap, os::raw::c_char};

pub enum Action {
    Pressed,
    Released,
}

pub type Binding = &'static str;
#[derive(Default)]
pub struct Input {
    axis: HashMap<Binding, f32>,
    action: HashMap<Binding, Action>,

    action_bindings: Vec<Binding>,
    axis_bindings: Vec<Binding>,
}
impl Input {
    pub fn get_mouse_delta(&self) -> (f32, f32) {
        let mut x = 0.0;
        let mut y = 0.0;

        unsafe {
            (bindings().get_mouse_delta)(&mut x, &mut y);
        }
        (x, y)
    }
    pub fn register_action_binding(&mut self, binding: Binding) {
        self.action_bindings.push(binding);
    }
    pub fn register_axis_binding(&mut self, binding: Binding) {
        self.axis_bindings.push(binding);
    }

    pub fn update(&mut self) {
        self.axis.clear();

        for binding in &self.action_bindings {
            let check_state = |state: ActionState| -> bool {
                let mut out = 0;
                unsafe {
                    (bindings().get_action_state)(
                        binding.as_ptr() as *const c_char,
                        binding.len(),
                        state,
                        &mut out,
                    );
                }
                out == 1
            };

            if check_state(ActionState::Pressed) {
                self.action.insert(binding, Action::Pressed);
            }
            if check_state(ActionState::Released) {
                self.action.insert(binding, Action::Released);
            }
        }
        for binding in &self.axis_bindings {
            let mut value: f32 = 0.0;
            unsafe {
                (bindings().get_axis_value)(
                    binding.as_ptr() as *const c_char,
                    binding.len(),
                    &mut value,
                );
            }
            self.axis.insert(binding, value);
        }
    }

    pub fn get_axis_value(&self, binding: Binding) -> Option<f32> {
        self.axis.get(&binding).copied()
    }

    pub fn is_action_pressed(&self, binding: Binding) -> bool {
        self.action
            .get(binding)
            .map_or(false, |state| matches!(state, Action::Pressed))
    }
    pub fn is_action_released(&self, binding: Binding) -> bool {
        self.action
            .get(binding)
            .map_or(false, |state| matches!(state, Action::Released))
    }
}
