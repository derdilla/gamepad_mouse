mod mouse;

use std::{thread, time::Duration};

use dialoguer::FuzzySelect;
use enigo::{Enigo, Keyboard, Mouse, Settings};
use gilrs::{Event, EventType::*, Gamepad, GamepadId, Gilrs};

use mouse::*;

fn select_gamepad(gilrs: &Gilrs) -> Option<(GamepadId, Gamepad)> {
    let gamepads: Vec<(GamepadId, Gamepad<'_>)> = gilrs.gamepads().collect();

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

struct ButtonMapper {
    enigo: Enigo,
    /// Multiplied with values in [0..1] to determine the next relative mouse position.
    speed_mod: f32,
    mouse_mover: MouseMoverHandle,
    // store the offset of the last knob movement to fix some controllers not returning
    // a definite zero value when letting go.
    last_x: f32,
    last_y: f32,
}

impl ButtonMapper {
    pub fn new() -> Self {
        let mouse_mover = MouseMoverHandle::get();
        ButtonMapper {
            enigo: Enigo::new(&Settings::default()).unwrap(),
            speed_mod: 2.0,
            mouse_mover,
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    #[rustfmt::skip]
    pub fn handle_event(&mut self, event: gilrs::EventType) {
        let performed_action = match event {
            // enter (ABXY on gamepad)
            ButtonPressed(gilrs::Button::South, _) => self.enigo.key(enigo::Key::Return, enigo::Direction::Press),
            ButtonReleased(gilrs::Button::South, _) => self.enigo.key(enigo::Key::Return, enigo::Direction::Release),
            ButtonPressed(gilrs::Button::East | gilrs::Button::North | gilrs::Button::West, _) => Ok(()),
            ButtonReleased(gilrs::Button::East | gilrs::Button::North | gilrs::Button::West, _) => Ok(()),

            // arrow keys
            ButtonPressed(gilrs::Button::DPadUp, _) => self.enigo.key(enigo::Key::UpArrow, enigo::Direction::Press),
            ButtonReleased(gilrs::Button::DPadUp, _) => self.enigo.key(enigo::Key::UpArrow, enigo::Direction::Release),
            ButtonPressed(gilrs::Button::DPadRight, _) => self.enigo.key(enigo::Key::RightArrow, enigo::Direction::Press),
            ButtonReleased(gilrs::Button::DPadRight, _) => self.enigo.key(enigo::Key::RightArrow, enigo::Direction::Release),
            ButtonPressed(gilrs::Button::DPadDown, _) => self.enigo.key(enigo::Key::DownArrow, enigo::Direction::Press),
            ButtonReleased(gilrs::Button::DPadDown, _) => self.enigo.key(enigo::Key::DownArrow, enigo::Direction::Release),
            ButtonPressed(gilrs::Button::DPadLeft, _) => self.enigo.key(enigo::Key::LeftArrow, enigo::Direction::Press),
            ButtonReleased(gilrs::Button::DPadLeft, _) => self.enigo.key(enigo::Key::LeftArrow, enigo::Direction::Release),
            ButtonPressed(gilrs::Button::Start, _) => self.enigo.key(enigo::Key::Meta, enigo::Direction::Press),

            // central keys
            ButtonReleased(gilrs::Button::Start, _) => self.enigo.key(enigo::Key::Meta, enigo::Direction::Release),
            
            // mouse (trigger is the small button trigger2 the big, lower one)
            ButtonChanged(gilrs::Button::LeftTrigger, v, _) => self.enigo.button(enigo::Button::Back, Self::dir(v)),
            ButtonChanged(gilrs::Button::LeftTrigger2, v, _) => self.enigo.button(enigo::Button::Left, Self::dir(v)),
            ButtonChanged(gilrs::Button::RightTrigger, v, _) => self.enigo.button(enigo::Button::Forward, Self::dir(v)),
            ButtonChanged(gilrs::Button::RightTrigger2, v, _) => self.enigo.button(enigo::Button::Right, Self::dir(v)),

            // mouse movement
            AxisChanged(gilrs::Axis::LeftStickX, mut v, _) => {
                if (v.abs() < 0.010)
                    || (v.abs() < 0.04 && v.abs() < self.last_x.abs()) {
                    v = 0.0;
                }
                self.last_x = v;
                self.mouse_mover.set_target_vel_x(self.speed_mod * v); Ok(()) },
            AxisChanged(gilrs::Axis::LeftStickY, mut v, _) => {
                if (v.abs() < 0.015)
                    || (v.abs() < 0.05 && v.abs() < self.last_y.abs()) {
                    v = 0.0;
                }
                self.last_y = v;
                self.mouse_mover.set_target_vel_y(-1.0 * self.speed_mod * v); Ok(()) },
            _ => Ok(()),
        };
        if let Err(err) = performed_action {
            println!("Failed to simulate action: {:?}", err);
        }
    }

    /// Determine whether to press or release from button presse strenght (0-1).
    fn dir(v: f32) -> enigo::Direction {
        assert!((0.0..=1.0).contains(&v));
        if v >= 0.5 {
            enigo::Direction::Press
        } else {
            enigo::Direction::Release
        }
    }
}

fn main() {
    let mut gilrs = Gilrs::new().unwrap();
    if let Some((gamepadid, gamepad)) = select_gamepad(&gilrs) {
        println!("Selected: {}", &gamepad.name());
        let mut mapper = ButtonMapper::new();
        loop {
            //println!("{:?}", &gamepad);
            while let Some(Event {
                id, event, ..
            }) = gilrs.next_event()
            {
                if id != gamepadid {
                    continue;
                }
                mapper.handle_event(event);

                //println!("{:?} New event: {:?}", time, event);
            }
            thread::sleep(Duration::from_millis(100))
        }
    }
}
