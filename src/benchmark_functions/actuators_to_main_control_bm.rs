use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::time::Duration;
use scheduled_thread_pool::ScheduledThreadPool;
use crate::actuator::sprayers;
use crate::rabbitmq_functions::send;
use crate::main_control::main_control;
use crate::benchmark_functions::main_control_bm::MAIN_CONTROL_OPERATING;
use crate::rabbitmq_functions::receive;
use crate::structs::{GPS, Mts_Camera, Ultrasonic};


pub static WATER_DIS_OPERATING: Mutex<bool> = Mutex::new(false);
pub static WATER_TANK_LEVEL: Mutex<i32> = Mutex::new(100);

pub static FERTILISER_DIS_OPERATING: Mutex<bool> = Mutex::new(false);
pub static FERTILISER_TANK_LEVEL: Mutex<i32> = Mutex::new(100);

pub static PESTICIDE_DIS_OPERATING: Mutex<bool> = Mutex::new(false);
pub static PESTICIDE_TANK_LEVEL: Mutex<i32> = Mutex::new(100);


pub fn bm_actuators_to_main_control() {
    let(soil_moist_s, soil_moist_r) = channel();
    let(soil_nutr_s, soil_nutr_r) = channel();
    let(thermal_s, thermal_r) = channel();

    // Receive command from the main control
    let instruc = "Off water dispenser".to_string();
    // Send the data through the sender channel
    soil_moist_s.send(instruc).unwrap();
    // Call the actuator function
    water_dispenser(&soil_moist_r);

    let instruc = "Maintain".to_string();
    // Send the data through the sender channel
    soil_nutr_s.send(instruc).unwrap();
    // Call the actuator function
    fertiliser_dispenser(&soil_nutr_r);

    // Receive command from the main control
    let instruc = "On pesticide dispensing".to_string();
    // Send the data through the sender channel
    thermal_s.send(instruc).unwrap();
    // Call the actuator function
    pesticide_dispenser(&thermal_r);

    main_control_data_receiver();
}

pub fn water_dispenser(instruc_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match instruc_r.try_recv() {
        Ok(instruc) => {
            // Lock the mutexes
            let mut operating = WATER_DIS_OPERATING.lock().unwrap();
            let mut tank_level = WATER_TANK_LEVEL.lock().unwrap();
            let mut remaining_lvl = *tank_level;

            // Handle different instructions
            if instruc == "Off water dispenser" {
                *operating = false; // Stop the water dispenser
            } else if instruc == "On water dispenser" {
                *operating = true; // Start the water dispenser
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10
            } else if instruc == "Maintain" && *operating {
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10 if operating
            }

            // Update the tank level
            *tank_level = remaining_lvl;

            // Send the remaining water level to the main control
            let water_disp_queue = "water_disp_mc".to_string();
            let _ = send(remaining_lvl.to_string(), &water_disp_queue);
        }
        Err(_) => {}
    }
}


pub fn fertiliser_dispenser(instruc_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match instruc_r.try_recv() {
        Ok(instruc) => {
            // Lock the mutexes
            let mut operating = FERTILISER_DIS_OPERATING.lock().unwrap();
            let mut tank_level = FERTILISER_TANK_LEVEL.lock().unwrap();
            let mut remaining_lvl = *tank_level;

            // Handle different instructions
            if instruc == "Off fertiliser dispensing" {
                *operating = false; // Stop the fertiliser dispenser
            } else if instruc == "On fertiliser dispensing" {
                *operating = true; // Start the fertiliser dispenser
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10
            } else if instruc == "Maintain" && *operating {
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10 if operating
            }

            // Update the tank level
            *tank_level = remaining_lvl;

            // Send the remaining fertiliser level to the main control
            let fert_disp_queue = "fert_disp_mc".to_string();
            let _ = send(remaining_lvl.to_string(), &fert_disp_queue);
        }
        Err(_) => {}
    }
}


pub fn pesticide_dispenser(instruc_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match instruc_r.try_recv() {
        Ok(instruc) => {
            // Lock the mutexes
            let mut operating = PESTICIDE_DIS_OPERATING.lock().unwrap();
            let mut tank_level = PESTICIDE_TANK_LEVEL.lock().unwrap();
            let mut remaining_lvl = *tank_level;

            // Handle different instructions
            if instruc == "Off pesticide dispensing" {
                *operating = false; // Stop the pesticide dispenser
            } else if instruc == "On pesticide dispensing" {
                *operating = true; // Start the pesticide dispenser
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10
            } else if instruc == "Maintain" && *operating {
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10 if operating
            }

            // Update the tank level
            *tank_level = remaining_lvl;

            // Send the remaining pesticide level to the main control
            let pest_disp_queue = "pest_disp_mc".to_string();
            let _ = send(remaining_lvl.to_string(), &pest_disp_queue);
        }
        Err(_) => {}
    }
}

pub fn operation_suspend() {
    // Lock the mutexes
    let mut w_operating = WATER_DIS_OPERATING.lock().unwrap();
    let mut f_operating = FERTILISER_DIS_OPERATING.lock().unwrap();
    let mut p_operating = PESTICIDE_DIS_OPERATING.lock().unwrap();

    // Set the operating status of all dispensers to false
    *w_operating = false;
    *f_operating = false;
    *p_operating = false;
}

pub fn main_control_data_receiver() {
    // Create a thread pool with 8 threads
    let pool = ScheduledThreadPool::new(3);

    // Water dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the water dispenser
            let water_tank_lvl = receive("water_disp_mc");
        }
    );

    // Fertiliser dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the fertiliser dispenser
            let fert_tank_lvl = receive("fert_disp_mc");
        }
    );

    // Pesticide dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the pesticide dispenser
            let pest_tank_lvl = receive("pest_disp_mc");
        }
    );
}