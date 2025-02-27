use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, SystemTime},
};

use enigo::{Enigo, Mouse};

#[derive(Debug)]
pub(crate) struct MouseMover {
    max_change_per_milli: f32,
    mouse_speed_mod: f32,
    last_tick: SystemTime,
    target_vel_x: f32,
    current_vel_x: f32,
    target_vel_y: f32,
    current_vel_y: f32,
    enigo: Enigo,
}

impl MouseMover {
    pub fn new() -> MouseMover {
        MouseMover {
            // whatever constant feels good.
            max_change_per_milli: 0.3,
            mouse_speed_mod: 2.0,
            last_tick: SystemTime::now(),
            target_vel_x: 0.0,
            current_vel_x: 0.0,
            target_vel_y: 0.0,
            current_vel_y: 0.0,
            enigo: Enigo::new(&enigo::Settings::default()).unwrap(),
        }
    }

    pub fn start(mut self) -> MouseMoverHandle {
        let (handler, recv) = MouseMoverHandle::new();
        thread::spawn(move || {
            loop {
                if let Ok((x, y)) = recv.recv_timeout(Duration::from_nanos(1000)) {
                    self.target_vel_x = x * self.mouse_speed_mod;
                    self.target_vel_y = y * self.mouse_speed_mod;
                }
                self.tick();
                thread::sleep(Duration::from_millis(1));

                if self.target_vel_x == 0.0
                    && self.current_vel_x == 0.0
                    && self.target_vel_y == 0.0
                    && self.current_vel_y == 0.0
                {
                    // Wait for next message
                    match recv.recv() {
                        Ok((x, y)) => {
                            self.target_vel_x = x;
                            self.target_vel_y = y;
                        },
                        Err(_) => break,
                    }
                }
            }
        });
        handler
    }

    fn tick(&mut self) {
        if self.target_vel_x == 0.0
            && self.current_vel_x == 0.0
            && self.target_vel_y == 0.0
            && self.current_vel_y == 0.0
        {
            self.last_tick = SystemTime::now();
            return;
        }

        let now = SystemTime::now();
        let elapse_millis = now
            .duration_since(self.last_tick)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        let max_change =
            (((self.max_change_per_milli as f64) * (elapse_millis as f64)) as f32).abs();

        // Calculate a factor no higher than [max_change_per_nanno] to get from [curr] to [target].
        self.current_vel_x += (self.target_vel_x - self.current_vel_x)
            .min(max_change)
            .max(-1.0 * max_change);
        self.current_vel_y += (self.target_vel_y - self.current_vel_y)
            .min(max_change)
            .max(-1.0 * max_change);

        // hack for faster slowdowns
        if self.target_vel_x == 0.0 {
            self.current_vel_x /= 2.0;
        }
        if self.target_vel_y == 0.0 {
            self.current_vel_y /= 2.0;
        }

        self.last_tick = now;
        self.enigo
            .move_mouse(
                self.current_vel_x.round() as i32,
                self.current_vel_y.round() as i32,
                enigo::Coordinate::Rel,
            )
            .expect("Unable to move mouse.");
    }
}

pub struct MouseMoverHandle {
    last_x: f32,
    last_y: f32,
    sender: Sender<(f32, f32)>,
}

impl MouseMoverHandle {
    pub fn get() -> Self {
        MouseMover::new().start()
    }

    pub fn new() -> (Self, Receiver<(f32, f32)>) {
        let (send, rec) = mpsc::channel();
        (
            MouseMoverHandle {
                last_x: 0.0,
                last_y: 0.0,
                sender: send,
            },
            rec,
        )
    }

    pub fn move_x(&mut self, mut x: f32) -> bool {
        if (x.abs() < 0.005)
            || (x.abs() < 0.05 && x.abs() < self.last_x.abs()) {
            x = 0.0;
        }
        self.last_x = x*0.6;
        self.send()
    }

    pub fn move_y(&mut self, mut y: f32) -> bool {
        if (y.abs() < 0.005)
            || (y.abs() < 0.05 && y.abs() < self.last_y.abs()) {
            y = 0.0;
        }
        self.last_y = -y*0.6;
        self.send()
    }

    fn send(&mut self) -> bool {
        self.sender
            .send((self.last_x, self.last_y))
            .is_ok()
    }
}
