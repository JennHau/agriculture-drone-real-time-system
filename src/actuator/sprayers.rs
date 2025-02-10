use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use crate::rabbitmq_functions::send;
use crate::main_control::main_control;
use crate::rabbitmq_functions::receive;


pub static WATER_DIS_OPERATING: Mutex<bool> = Mutex::new(false);
pub static WATER_TANK_LEVEL: Mutex<i32> = Mutex::new(100);

pub static FERTILISER_DIS_OPERATING: Mutex<bool> = Mutex::new(false);
pub static FERTILISER_TANK_LEVEL: Mutex<i32> = Mutex::new(100);

pub static PESTICIDE_DIS_OPERATING: Mutex<bool> = Mutex::new(false);
pub static PESTICIDE_TANK_LEVEL: Mutex<i32> = Mutex::new(100);

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
                println!("Water Dispenser: Stop operating");
                *operating = false; // Stop the water dispenser
            } else if instruc == "On water dispenser" {
                println!("Water Dispenser: Start operating");
                *operating = true; // Start the water dispenser
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10
            } else if instruc == "Maintain" && *operating {
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10 if operating
                println!("Water Dispenser: Dispensing water");
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
                println!("Fertiliser Dispenser: Stop operating");
                *operating = false; // Stop the fertiliser dispenser
            } else if instruc == "On fertiliser dispensing" {
                println!("Fertiliser Dispenser: Start operating");
                *operating = true; // Start the fertiliser dispenser
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10
            } else if instruc == "Maintain" && *operating {
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10 if operating
                println!("Fertiliser Dispenser: Dispensing fertiliser")
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
                println!("Pesticide Dispenser: Stop operating")
            } else if instruc == "On pesticide dispensing" {
                *operating = true; // Start the pesticide dispenser
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10
                println!("Pesticide Dispenser: Start operating")
            } else if instruc == "Maintain" && *operating {
                remaining_lvl = *tank_level - 10; // Decrease the tank level by 10 if operating
                println!("Pesticide Dispenser: Dispensing pesticide")
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

    println!("Sprayers [Water, Fertiliser, & Pesticide Dispenser] operations temporarily suspended.")
}