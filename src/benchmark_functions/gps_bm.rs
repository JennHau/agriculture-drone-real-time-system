use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, MutexGuard};
use rand::Rng;
use crate::main_control::main_control::{BACK_HOME, RESUME_OPERATION, TASK_ONGOING};
use crate::structs::GPS;

use crate::main_control::main_control;

// Define a static variable to hold the current GPS position
static CURRENT_POSITION: Mutex<GPS> = Mutex::new(GPS {
    longitude: 5,
    latitude: 100,
});

pub fn bm_gps() {
    let(gps_sender, gps_recv) = channel();
    generate_gps_data(&gps_sender);
}

// Function to generate GPS data
pub fn generate_gps_data(gps_sender: &Sender<GPS>) {
    // Lock and access mutexes
    let mut current_position: MutexGuard<GPS> = CURRENT_POSITION.lock().unwrap();

    // Create a random number generator
    let mut rng = rand::thread_rng();

    // Generate a random number between 0 and 9
    let prob = rng.gen_range(0..10);

    // Define the ending longitude value for the task
    let longitude_ending = 130;

    // Clone the task ongoing status and back home status
    let task_ongoing = TASK_ONGOING.lock().unwrap().clone();
    let back_home = BACK_HOME.lock().unwrap().clone();

    // If the task is ongoing and the drone is not returning home
    if task_ongoing && !back_home {
        if prob < 7 {
            // 70% probability of staying on the straight line
            if current_position.latitude != longitude_ending {
                let resume = RESUME_OPERATION.lock().unwrap();
                if !*resume {
                    current_position.latitude += 5; // Move five unit up
                } else {
                    current_position.latitude += 1; // Move one unit up
                }
            }
            // Send the updated GPS position
            gps_sender.send(current_position.clone()).unwrap();
        } else {
            // 30% probability of deviation
            let deviation = rng.gen_range(-3..5); // Allow deviation of up to 5 units in any direction
            if current_position.latitude != longitude_ending {
                // Lock and access mutex
                let resume = RESUME_OPERATION.lock().unwrap();
                // If resume operation is false
                if !*resume {
                    current_position.latitude += 5; // Move five unit up
                } else {
                    current_position.latitude += 1; // Move one unit up
                }
            }
            // Apply the deviation to the longitude
            current_position.longitude += deviation;
            // Send the updated GPS position with deviation
            gps_sender.send(current_position.clone()).unwrap();
            // Revert the deviation to longitude
            current_position.longitude -= deviation;
        };
    } else if back_home {
        // If the drone is returning home
        if prob < 7 {
            // 70% probability of staying on the straight line
            if current_position.latitude > 100 {
                current_position.latitude -= 2; // Move two unit down
                if current_position.latitude < 100 {
                    // Ensure the latitude does not go below 100
                    current_position.latitude = 100;
                }
            }
            // Send the updated GPS position
            gps_sender.send(current_position.clone()).unwrap();
        } else {
            // 30% probability of deviation
            let deviation = rng.gen_range(-3..5); // Allow deviation of up to 5 units in any direction
            if current_position.latitude != 100 {
                current_position.latitude -= 2; // Move two unit down
                if current_position.latitude < 100 {
                    // Ensure the latitude does not go below 100
                    current_position.latitude = 100;
                }
            }
            // Apply the deviation to the longitude
            current_position.longitude += deviation;
            // Send the updated GPS position with deviation
            gps_sender.send(current_position.clone()).unwrap();
            // Revert the deviation to longitude
            current_position.longitude -= deviation;
        };
    }
}