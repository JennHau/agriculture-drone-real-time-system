use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::structs::Ultrasonic;

use crate::main_control::main_control;
static ALTITUDE: Mutex<i32> = Mutex::new(0);

pub static REACHED_MAX_ALTITUDE: Mutex<bool> = Mutex::new(false);


pub fn bm_ultrasonic_sensor() {
    let(ultra_sensor_sender, ultra_sensor_recv) = channel();
    generate_ultrasonic_data(&ultra_sensor_sender);
}

// Function to generate ultrasonic sensor data
pub fn generate_ultrasonic_data(ultra_sensor_sender: &Sender<Ultrasonic>) {
    // Create a random number generator
    let mut rng = rand::thread_rng();

    // Generate a random number between 0 and 9
    let prob = rng.gen_range(0..10);

    // Determine the distance based on a probability
    let distance = if prob < 7 {
        // 70% probability
        rng.gen_range(101..200) // Clear range
    } else {
        // 30% probability
        rng.gen_range(50..101) // Detected range
    };


    // Lock and access the mutexes
    let mut reached_max_altitude = REACHED_MAX_ALTITUDE.lock().unwrap();
    let mut altitude = ALTITUDE.lock().unwrap();
    let mut landing = main_control::LANDING.lock().unwrap();

    if !*landing {
        // If drone not landing
        if !*reached_max_altitude {
            // If drone has not reach max altitude
            *altitude += 5; // Increase altitude by 5 meters per second

            // Check if altitude has reached 50 meters
            if *altitude == 50 {
                *reached_max_altitude = true;
            }
        } else {
            // Once max altitude is reached, maintain it with occasional deviations
            let prob = rng.gen_range(0..10); // Generate a random number between 0 and 9
            if prob < 7 {
                // 70% probability
                *altitude = 50; // Maintain altitude at 50 meters
            } else {
                // 30% probability for altitude deviations within a range of -5 to 5 meters
                let deviation = rng.gen_range(-5..=5);
                *altitude = 50 + deviation; // Apply the deviation to the altitude
            };
        };

    } else {
        // If landing, decrease altitude by 10 meters per second
        *altitude -= 10;
        if *altitude < 0 {
            *altitude = 0; // Ensure altitude does not go below 0
        }
    }

    // Create an Ultrasonic data struct with the calculated distance and altitude
    let ultra_data = Ultrasonic {
        obstacles_dtc: distance,
        altitude: *altitude
    };

    // Send the generated data through the provided sender channel
    ultra_sensor_sender.send(ultra_data).unwrap();
}