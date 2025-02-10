use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use crate::rabbitmq_functions::receive;

pub static GEARUP: Mutex<bool> = Mutex::new(false);

pub fn gear(instruc_r: &Receiver<String>) {
    // Attempt to receive instructions from the receiver channel
    match instruc_r.try_recv() {
        Ok(instruc) => {
            // Lock the mutex to access and modify the gear_up status
            let mut gear_up = GEARUP.lock().unwrap();

            // Check the received instruction and update the gear status accordingly
            if instruc == "Retract gear" {
                *gear_up = true;
            } else if instruc == "Deploy gear" {
                *gear_up = false;
            }
        }
        Err(_) => {}
    }
}

