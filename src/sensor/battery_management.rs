use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Duration;
use scheduled_thread_pool::ScheduledThreadPool;
use crate::main_control::main_control::{RESUME_OPERATION, TASK_ONGOING};

// Define a static variable to hold the battery level
pub static BATTERY_LEVEL: Mutex<i32> = Mutex::new(105);


pub fn generate_battery_level(batt_sender: &Sender<i32>) {
    // Acquire the mutex lock to access the battery level
    let mut battery_level = BATTERY_LEVEL.lock().unwrap();
    let mut resume = RESUME_OPERATION.lock().unwrap();
    let mut task_ongoing = TASK_ONGOING.lock().unwrap();


    if *battery_level < 21 {
        *battery_level -= 1;
        // Ensure battery level doesn't go 0
        if *battery_level <= 2 {
            *battery_level = 2;
        }
    } else if !*resume && *task_ongoing {
        *battery_level -= 1;
    } else {
        // Reduce battery level
        *battery_level -= 5;
    }

    // Send battery level through the channel
    batt_sender.send(*battery_level).unwrap();
}