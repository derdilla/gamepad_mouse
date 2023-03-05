use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use byteorder::{BigEndian, ReadBytesExt};
use enigo::*;

static MOVE_MULTIPLIER: f64 = 20.;
static SCROLL_MUTLIPLIER: f64 = 1.5;
static CLAMP_THRESHOLD: f64 = 0.04;

struct State {
    running: bool,
    x: i16,
    y: i16,
    rx: f64,
    ry: f64,
    z: i16,
    rz: i16
}

fn get_gamepad_handler() -> String {
    // 1. open device file
    // 2. read 2nd line and then evry 10 lines further to get device name. find a name containing 'gamepad' (case insensitive)
    // 3. 4 lines below are the Handlers. After the equal sign are space speperated handlers; we want the one containing event
    // 4. return the handlercontaining event
    let f = File::open("/proc/bus/input/devices").unwrap();
    let reader = BufReader::new(f);

    let mut lines = reader.lines();

    while !lines.next().unwrap().unwrap().to_lowercase().contains("gamepad") {
    }

    let handlers_line = lines.nth(3).unwrap().unwrap();
    let mut handlers = handlers_line.split("=").nth(1).unwrap().split(" ");
    
    let y = handlers.find(|x| x.contains("event")).unwrap().to_string();
    y.clone()
}

fn main() -> std::io::Result<()> {
    

    let mut file_location = "/dev/input/".to_owned();
    let eventfile_name = get_gamepad_handler();
    file_location.push_str(&eventfile_name);
    println!("{}", file_location);
    let f = File::open(file_location)?;
    let mut reader = BufReader::new(f);

    let state = Arc::new(Mutex::new(State {
        running: true,
        x: 0,
        y: 0,
        rx: 0.,
        ry: 0.,
        z: 0,
        rz: 0,
    }));

    let poll_state = Arc::clone(&state);

    let poll = thread::spawn(move || {
        loop {
            // read from file
            let mut buf = vec![];
            let mut chunk = (&mut reader).take(24);
            chunk.read_to_end(&mut buf).expect("Didn't read enough");

            // parse result
            /*
            Event code 3 (ABS_RX)
                Value    128
                Min   -32768
                Max    32767
                Fuzz      16
                Flat     128
            Event code 4 (ABS_RY)
                Value   -129
                Min   -32768
                Max    32767
                Fuzz      16
                Flat     128
            */
            let mut tmp = Cursor::new(buf[21..23].to_vec()); // is this the corect part of array?
            let value = tmp.read_i16::<BigEndian>().unwrap(); // only works for joystics

            let mut code:u16 = buf[19].into();
            code *= 256;
            code += buf[18] as u16;

            //println!("{:?}", buf);

            let mut state = poll_state.lock().unwrap();
            match code {
                0 => { // ABS_X
                    state.x = value;
                }
                1 => { // ABS_Y
                    state.y = value;
                }
                3 => { // ABS_RX
                    let mut value = (value as f64)/32768.; // todo: move logic
                    if value.abs() < CLAMP_THRESHOLD { // prevent mouse movent despite unmoved joystic
                        value = 0.;
                    }
                    state.rx = value;
                }
                4 => { // ABS_RY
                    let mut value = (value as f64)/32768.;  // todo: move logic
                    if value.abs() < CLAMP_THRESHOLD { // prevent mouse movent despite unmoved joystic
                        value = 0.;
                    }
                    state.ry = value;
                }
                2 => { // ABS_Z
                    state.z = buf[20].into(); 
                }
                5 => { // ABS_RZ
                    state.rz = buf[20].into(); 
                }
                314 => { // BTN_SELECT
                    state.running = false;
                }
                _ => {
                    println!("{}", code)
                }
            }
            if !state.running { 
                break;
            }
        }
    });

    let update_state = Arc::clone(&state);
    let update = thread::spawn(move ||{
        let mut last_ex = SystemTime::now();
        let mut enigo = Enigo::new();
        
        loop {
            if last_ex.elapsed().unwrap().as_millis() > 10 {
                last_ex = SystemTime::now();

                let state = update_state.lock().unwrap();
                if !state.running { 
                    break;
                }

                // move mouse
                enigo.mouse_move_relative((state.rx*MOVE_MULTIPLIER) as i32, 0);
                enigo.mouse_move_relative(0, (state.ry*MOVE_MULTIPLIER) as i32);

                // mouse buttons
                if state.rz > 128 {
                    enigo.mouse_down(MouseButton::Right);
                } else if state.rz > 0 {
                    enigo.mouse_up(MouseButton::Right);
                }
                if state.z > 128 {
                    enigo.mouse_down(MouseButton::Left);
                } else if state.z > 0 {
                    enigo.mouse_up(MouseButton::Left);
                }

                // scroll
                let y = (state.y as f64)/32768.;
                enigo.mouse_scroll_y((y*SCROLL_MUTLIPLIER) as i32);
            }
        }
    });

    poll.join().unwrap();
    update.join().unwrap();



        
    Ok(())
}