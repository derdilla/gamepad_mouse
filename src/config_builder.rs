use enigo::Enigo;
use gilrs::Gilrs;
use crate::config::{Config, Keymap, Stick};
use crate::mouse::MouseMover;

pub struct ConfigBuilder {
    gamepad: gilrs::GamepadId,
    keymap: crate::config::Keymap,
    /// The stick that controls the mouse movement.
    mouse_stick: crate::config::Stick,
}

impl ConfigBuilder {
    /// Create a config builder with sane defaults, if a gamepad is connected.
    pub fn default() -> Option<Self> {
        let gilrs = Gilrs::new().ok()?;
        let gamepad = gilrs
            .gamepads()
            .find(|(id, _)| gilrs.connected_gamepad(*id).is_some())
            .map(|(id, _)| id)?;
        Some(ConfigBuilder {
            gamepad,
            keymap: Keymap::default(),
            mouse_stick: Stick::Left,
        })
    }

    pub fn from_file() -> Option<Self> {
        todo!()
    }

    pub fn from_interactive() -> Option<Self> {
        todo!()
    }

    pub fn from_cli() -> Option<Self> {
        todo!()
    }

    pub fn build<'a>(self) -> Option<Config> {
        let enigo = Enigo::new(&enigo::Settings::default()).ok()?;
        Some(Config::new(
            self.gamepad,
            self.keymap,
            self.mouse_stick,
            enigo,
            MouseMover::new().start(),
        ))
    }
}