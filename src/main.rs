mod mouse;
mod config;
mod config_builder;

use std::{thread, time::Duration};

use gilrs::{Event, Gilrs};

use crate::config_builder::ConfigBuilder;

fn main() {
    let mut gilrs = Gilrs::new().unwrap();
    let mut mapper = ConfigBuilder::default().unwrap().build().unwrap();
    loop {
        while let Some(Event {
                           id, event, ..
                       }) = gilrs.next_event() {
            mapper.handle(id, &event);

        }
        thread::sleep(Duration::from_millis(100))
    }
}
