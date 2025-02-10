use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use rand::Rng;

static HIGH_THERMAL: Mutex<bool> = Mutex::new(false);
static THERMAL_VALUE: Mutex<i32> = Mutex::new(0);

pub fn bm_thermal_sensor() {
    let(thermal_sender, thermal_recv) = channel();
    generate_thermal_data(&thermal_sender);
}

// Function to generate thermal sensor data
pub fn generate_thermal_data(thermal_sender: &Sender<i32>) {
    // Lock and access mutexes
    let mut high_termal = HIGH_THERMAL.lock().unwrap();
    let mut thermal_value = THERMAL_VALUE.lock().unwrap();

    // Create a random number generator
    let mut rng = rand::thread_rng();

    // Generate a random number between 0 and 9
    let prob = rng.gen_range(0..10);

    if prob < 7 && !*high_termal {
        // 70% probability
        *thermal_value = rng.gen_range(20..31); // Moderate temperature range

    } else if prob > 6 && !*high_termal {
        // 30% probability for temperatures outside the normal range
        *thermal_value = rng.gen_range(31..40); // Generate temperatures above the normal range
        *high_termal = true; // Set the high thermal flag to true

    } else if *high_termal{
        // If the high thermal flag is true, decrease the temperature gradually
        *thermal_value -= 2; // Decrease temperature by 2
        if *thermal_value < 31 {
            *high_termal = false; // Reset the high thermal flag if the temperature goes below 31
        }
    };

    // Send the generated thermal value through the provided sender channel
    thermal_sender.send(*thermal_value).unwrap();
}