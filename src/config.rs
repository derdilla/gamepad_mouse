use std::collections::HashMap;

use crate::mouse::MouseMoverHandle;
use dialoguer::FuzzySelect;
use enigo::{Enigo, Keyboard, Mouse};

pub struct Config {
    gamepad: gilrs::GamepadId,
    keymap: Keymap,
    /// The stick that controls the mouse movement.
    mouse_stick: Stick,
    enigo: Enigo,
    mouse: MouseMoverHandle,

}

impl Config {
    pub(crate) fn new(gamepad_id: gilrs::GamepadId, keymap: Keymap, mouse_stick: Stick, enigo: Enigo, mouse_mover_handle: MouseMoverHandle) -> Self {
        Config {
            gamepad: gamepad_id,
            keymap,
            mouse_stick,
            enigo,
            mouse: mouse_mover_handle,
        }
    }

    pub fn handle(&mut self, id: gilrs::GamepadId, event: &gilrs::EventType) -> bool {
        if id != self.gamepad {
            return false;
        }
        self.keymap.handle_event(event, &mut self.enigo)
            || self.mouse_stick.handle_event(event, &mut self.mouse)
    }
}

pub(crate) struct Keymap {
    buttons: HashMap<gilrs::Button, enigo::Button>,
    keys: HashMap<gilrs::Button, enigo::Key>,
}

impl Keymap {
    /// Create a keymap with sane default keys
    pub(crate) fn default() -> Keymap {
        Keymap {
            buttons: HashMap::from([
                // mouse (trigger is the small button trigger2 the big, lower one)
                (gilrs::Button::LeftTrigger, enigo::Button::Back),
                (gilrs::Button::LeftTrigger2, enigo::Button::Left),
                (gilrs::Button::RightTrigger, enigo::Button::Forward),
                (gilrs::Button::RightTrigger2, enigo::Button::Right),
            ]),
            keys: HashMap::from([
                // enter (ABXY on gamepad)
                (gilrs::Button::South, enigo::Key::Return),
                // arrow keys
                (gilrs::Button::DPadUp, enigo::Key::UpArrow),
                (gilrs::Button::DPadRight, enigo::Key::RightArrow),
                (gilrs::Button::DPadDown, enigo::Key::DownArrow),
                (gilrs::Button::DPadLeft, enigo::Key::LeftArrow),
                // center keys
                (gilrs::Button::Start, enigo::Key::Meta), // "Start" mapped to Meta key
            ]),
        }
    }
}

impl Keymap {
    /// Handle button presses and return whether an event was handled.
    pub(crate) fn handle_event(&self, event: &gilrs::EventType, enigo: &mut enigo::Enigo) -> bool {
        if let gilrs::EventType::ButtonChanged(button, value, _) = event {
            if let Some(button) = self.buttons.get(button) {
                if *value > 0.5 {
                    enigo.button(button.clone(), enigo::Direction::Press).is_ok()
                } else {
                    enigo.button(button.clone(), enigo::Direction::Release).is_ok()
                }
            } else if let Some(key) = self.keys.get(button) {
                if *value > 0.5 {
                    enigo.key(key.clone(), enigo::Direction::Press).is_ok()
                } else {
                    enigo.key(key.clone(), enigo::Direction::Release).is_ok()
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub(crate) enum Stick {
    Left,
    Right,
}

impl Stick {
    pub(crate) fn handle_event(&self, event: &gilrs::EventType, mut mouse: &mut MouseMoverHandle) -> bool {
        let gilrs::EventType::AxisChanged(axis, value, _) = event else {
            return false;
        };
        match self {
            Stick::Left if *axis == gilrs::Axis::LeftStickX => mouse.move_x(*value),
            Stick::Right if *axis == gilrs::Axis::RightStickX => mouse.move_x(*value),
            Stick::Left if *axis == gilrs::Axis::LeftStickY => mouse.move_y(*value),
            Stick::Right if *axis == gilrs::Axis::RightStickY => mouse.move_y(*value),
            _ => false,
        }
    }
}

struct TUIConfigurator;

impl TUIConfigurator {
    fn select_gamepad(gilrs: &gilrs::Gilrs) -> Option<(gilrs::GamepadId, gilrs::Gamepad)> {
        let gamepads: Vec<(gilrs::GamepadId, gilrs::Gamepad<'_>)> = gilrs.gamepads().collect();

        if gamepads.is_empty() {
            println!("Couldn't detect any gamepads!");
            None
        } else if gamepads.len() == 1 {
            gamepads.first().map(|e| e.clone())
        } else {
            let gamepad_names = gamepads
                .iter()
                .map(|(id, pad)| format!("{} {}", &id, &pad.name()));
            let selection = FuzzySelect::new()
                .with_prompt("Multiple gamepads available:")
                .items(&gamepad_names.collect::<Vec<String>>())
                .interact()
                .unwrap();
            gamepads.get(selection).map(|e| e.clone())
        }
    }
}
