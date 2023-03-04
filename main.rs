use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::io::Cursor;
use std::ops::Mul;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use byteorder::{BigEndian, ReadBytesExt};
use enigo::*;

static MOVE_MULTI: f64 = 20.0;
static CLAMP_THRESHOLD: f64 = 0.04;

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

    
    let last_x = Arc::new(Mutex::new(0.));
    let last_y = Arc::new(Mutex::new(0.));

    let x1 = Arc::clone(&last_x);
    let y1 = Arc::clone(&last_y);

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
            let mut value = (tmp.read_i16::<BigEndian>().unwrap() as f64)/32768.;
            if value.abs() < CLAMP_THRESHOLD { // prevent mouse movent despite unmoved joystic
                value = 0.;
            }

            if buf[18] == 3 { // ABS_RX
                // println!("X: {}", value);
                let mut num = x1.lock().unwrap();
                *num = value;
            } else if buf[18] == 4 { // ABS_RY
                //println!("Y: {}", value);
                let mut num = y1.lock().unwrap();
                *num = value;               
            } else if buf[18] == 5 { // ABS_RZ
                
            }
        }
    });

    let x2 = Arc::clone(&last_x);
    let y2 = Arc::clone(&last_y);
    let update = thread::spawn(move ||{
        let mut last_ex = SystemTime::now();
        let mut enigo = Enigo::new();
        
        loop {
            if last_ex.elapsed().unwrap().as_millis() > 10 {
                last_ex = SystemTime::now();

                let x = x2.lock().unwrap();
                let y = y2.lock().unwrap();

                println!("X: {}\t|\tY: {}", *x, *y);
                //thread::sleep(time::Duration::from_millis(10));
                enigo.mouse_move_relative((x.clone()*MOVE_MULTI) as i32, 0);
                enigo.mouse_move_relative(0, (y.clone()*MOVE_MULTI) as i32);
            }

            
        }
    });

    poll.join().unwrap();
    update.join().unwrap();



        
    Ok(())
}