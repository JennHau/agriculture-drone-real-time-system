use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use once_cell::sync::Lazy;
use scheduled_thread_pool::ScheduledThreadPool;
use crate::actuator::sprayers::{FERTILISER_TANK_LEVEL, operation_suspend, PESTICIDE_TANK_LEVEL, WATER_DIS_OPERATING, WATER_TANK_LEVEL};
use crate::rabbitmq_functions::{receive, send};
use crate::structs::{GPS, Mts_Camera, Ultrasonic};

use crate::actuator::sprayers;
use crate::actuator::gear;

pub static TASK_ONGOING: Mutex<bool> = Mutex::new(true);
pub static BACK_HOME: Mutex<bool> = Mutex::new(false);
pub static LANDING: Mutex<bool> = Mutex::new(false);
pub static RESUME_OPERATION: Mutex<bool> = Mutex::new(true);
pub static LATEST_LATITUDE: Mutex<i32> = Mutex::new(100);

pub static MAIN_CONTROL_OPERATING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

pub fn data_receiver() {
    // Create channels for communication between threads
    let(mts_cam_sender, mts_cam_recv) = channel();
    let(thermal_sender, thermal_recv) = channel();
    let(gps_sender, gps_recv) = channel();
    let(ultra_sensor_sender, ultra_sensor_recv) = channel();
    let(batt_sender, batt_recv) = channel();
    let(water_lvl_sender, water_lvl_recv) = channel();
    let(fert_lvl_sender, fert_lvl_recv) = channel();
    let(pest_lvl_sender, pest_lvl_recv) = channel();

    // Create a thread pool with 8 threads
    let pool = ScheduledThreadPool::new(8);

    // Multispectral camera thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let data = receive("mts_cam");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                let mts_data:Mts_Camera = serde_json::from_str(&data).unwrap();
                // Send the data through the sender channel
                mts_cam_sender.send(mts_data).unwrap();
            }});

    // Thermal thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let thermal = receive("thermal");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                // Send the data through the sender channel
                thermal_sender.send(thermal).unwrap();
            }});


    // GPS thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let data = receive("gps");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                let gps:GPS = serde_json::from_str(&data).unwrap();
                // Send the data through the sender channel
                gps_sender.send(gps).unwrap();
            }});

    // Ultrasonic sensor thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let data = receive("ultra_sensor");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                let ultra_data:Ultrasonic = serde_json::from_str(&data).unwrap();
                // Send the data through the sender channel
                ultra_sensor_sender.send(ultra_data).unwrap();
            }});

    // Battery thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution\
        move ||{
            // Receive data from the sensor
            let batt = receive("batt");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                // Send the data through the sender channel
                batt_sender.send(batt).unwrap();
            }});

    // Water dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the water dispenser
            let water_tank_lvl = receive("water_disp_mc");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                // Send the data through the sender channel
                water_lvl_sender.send(water_tank_lvl).unwrap();
            }});

    // Fertiliser dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the fertiliser dispenser
            let fert_tank_lvl = receive("fert_disp_mc");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                // Send the data through the sender channel
                fert_lvl_sender.send(fert_tank_lvl).unwrap();
            }});

    // Pesticide dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the pesticide dispenser
            let pest_tank_lvl = receive("pest_disp_mc");
            // Check if the main control is operating
            if MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
                // Send the data through the sender channel
                pest_lvl_sender.send(pest_tank_lvl).unwrap();
            }});


    loop{
        // Process data from all sensors
        data_processor(&mts_cam_recv, &thermal_recv, &gps_recv, &ultra_sensor_recv, &batt_recv,
                       &water_lvl_recv, &fert_lvl_recv, &pest_lvl_recv);
        // Check if the main control is still operating
        if !MAIN_CONTROL_OPERATING.load(Ordering::Relaxed) {
            println!("Main Control is shut down.");
            break;
        }}}

fn data_processor(mts_cam_recv: &Receiver<Mts_Camera>, thermal_recv: &Receiver<String>,
                   gps_recv: &Receiver<GPS>, ultra_sensor_recv: &Receiver<Ultrasonic>,
                   batt_recv: &Receiver<String>, water_lvl_recv: &Receiver<String>,
                   fert_lvl_recv: &Receiver<String>, pest_lvl_recv: &Receiver<String>) {

    // Define channels for sending instructions to various actuators
    let soil_moist_queue = "soil_moist_act".to_string();
    let soil_nutr_queue = "soil_nutr_act".to_string();
    let thermal_queue = "thermal_act".to_string();
    let gps_queue = "gps_act".to_string();
    let obs_detc_queue = "obs_detc_act".to_string();
    let altitude_gear_queue = "altitude_gear_act".to_string();
    let altitude_motor_queue = "altitude_motor_act".to_string();
    let altitude_refill_queue = "altitude_refill_act".to_string();

    // Process data received from multispectral camera
    match mts_cam_recv.try_recv() {
        Ok(mts_data) => {
            // Print soil moisture and soil nutrient data
            println!("------------------------------[Soil Moisture: {}]", mts_data.soil_moist);
            println!("------------------------------[Soil Nutrient: {:.1}]", mts_data.soil_nutr);

            // Perform logic based on soil moisture and soil nutrient data
            let soil_moist_instruc = soil_moist_logic(mts_data.soil_moist);
            let soil_nutr_intruc = soil_nutr_logic(mts_data.soil_nutr);

            // Send instructions to respective queues
            let _ = send(soil_moist_instruc, &soil_moist_queue);
            let _ = send(soil_nutr_intruc, &soil_nutr_queue);
        } Err(_) => {}
    }

    // Process data received from thermal sensor
    match thermal_recv.try_recv() {
        Ok(thermal) => {
            // Convert string to integer
            match thermal.parse::<i32>() {
                Ok(temp) => {
                    // If parsing succeeds, print thermal data
                    println!("------------------------------[Thermal: {} Degree Celsius]", thermal);

                    // Perform logic based on thermal data
                    let thermal_instruc = thermal_logic(temp);

                    // Send instructions to respective queues
                    let _ = send(thermal_instruc, &thermal_queue);
                },
                Err(_) => {}
            }
        } Err(_) => {}
    }

    // Process data received from GPS sensor
    match gps_recv.try_recv() {
        Ok(gps) => {
            // Print GPS data
            println!("------------------------------[GPS: ({}, {})]", gps.longitude, gps.latitude);

            // Perform logic based on GPS data
            let gps_instruc = gps_logic(gps.longitude, gps.latitude);

            // Send instructions to GPS queue
            let _ = send(gps_instruc, &gps_queue);
        } Err(_) => {}
    }

    // Process data received from ultrasonic sensor
    match ultra_sensor_recv.try_recv() {
        Ok(ultra_data) => {
            // Print obstacles detection and altitude data
            println!("------------------------------[Obstacles Detection: {} metre]", ultra_data.obstacles_dtc);
            println!("------------------------------[Altitude: {} metre]", ultra_data.altitude);

            // Perform logic based on obstacles detection and altitude data
            let obst_detc_instruc = obst_dect_logic(ultra_data.obstacles_dtc);
            let altitude_gear_instruc = altitude_gear_logic(ultra_data.altitude);
            let altitude_motor_instruc = altitude_motor_logic(ultra_data.altitude);

            // Send instructions to respective queues
            let _ = send(obst_detc_instruc, &obs_detc_queue);
            let _ = send(altitude_gear_instruc, &altitude_gear_queue);
            let _ = send(altitude_motor_instruc, &altitude_motor_queue);
            let _ = send(ultra_data.altitude.to_string(), &altitude_refill_queue);
        } Err(_) => {}
    }

    // Process data received from battery sensor
    match batt_recv.try_recv() {
        Ok(batt) => {
            // Convert string into integer
            match batt.parse::<i32>() {
                Ok(temp) => {
                    // If parsing succeeds, print battery level
                    println!("------------------------------[Battery Level: {}%]", batt);
                    // Call battery_logic function
                    battery_logic(temp);
                },
                Err(_) => {}
            }
        } Err(_) => {}
    }

    // Process data received from water tank level sensor
    match water_lvl_recv.try_recv() {
        Ok(water_lvl) => {
            // Convert string into integer
            match water_lvl.parse::<i32>() {
                Ok(temp) => {
                    // If parsing succeeds, print water tank level
                    println!("------------------------------[Water Tank Level: {}%]", water_lvl);
                    //  Call water_lvl_logic function
                    water_lvl_logic(temp);
                },
                Err(_) => {}
            }
        } Err(_) => {}
    }

    // Process data received from fertiliser tank level sensor
    match fert_lvl_recv.try_recv() {
        Ok(fert_lvl) => {
            // Convert string into integer
            match fert_lvl.parse::<i32>() {
                Ok(temp) => {
                    // If parsing succeeds, print fertiliser tank level
                    println!("------------------------------[Fertiliser Tank Level: {}%]", fert_lvl);
                    //  Call fert_lvl_logic function
                    fert_lvl_logic(temp);
                },
                Err(_) => {}
            }
        } Err(_) => {}
    }

    // Process data received from pesticide tank level sensor
    match pest_lvl_recv.try_recv() {
        Ok(pest_lvl) => {
            // Convert string into integer
            match pest_lvl.parse::<i32>() {
                Ok(temp) => {
                    // If parsing succeeds, print pesticide tank level
                    println!("------------------------------[Pesticide Tank Level: {}%]", pest_lvl);
                    // Call the pesticide logic function
                    pest_lvl_logic(temp);
                },
                Err(_) => {}
            }
        } Err(_) => {}
    }
}

// Function to determine the action for controlling the water dispenser based on soil moisture level
fn soil_moist_logic(soil_moist: i32) -> String {
    // Lock the mutex to access the operating status of the water dispenser
    let mut operating = sprayers::WATER_DIS_OPERATING.lock().unwrap();

    // Determine the action based on the soil moisture level
    if soil_moist > 299 {
        if *operating {
            // If soil moisture is adequate and water dispenser is on, turn off the water dispenser
            "Off water dispenser".to_string()
        } else {
            // If soil moisture is adequate but water dispenser is already off
            // maintain current state
            "Maintain".to_string()
        }
    } else {
        if !*operating {
            // If soil moisture is low and water dispenser is off, turn on the water dispenser
            "On water dispenser".to_string()
        } else {
            // If soil moisture is low but water dispenser is already on, maintain current state
            "Maintain".to_string()
        }
    }
}

// Function to determine the action for controlling the fertilizer dispenser based on soil nutrient level
fn soil_nutr_logic(soil_nutr: f64) -> String {
    // Lock the mutex to access the operating status of the fertilizer dispenser
    let mut operating = sprayers::FERTILISER_DIS_OPERATING.lock().unwrap();

    // Determine the action based on the soil nutrient level
    if soil_nutr > 0.5 {
        if *operating {
            // If soil nutrient level is adequate and fertilizer dispenser is on
            // turn off the dispenser
            "Off fertiliser dispensing".to_string()
        } else {
            // If soil nutrient level is adequate but fertilizer dispenser is already off
            // maintain current state
            "Maintain".to_string()
        }
    } else {
        if !*operating {
            // If soil nutrient level is low and fertilizer dispenser is off, turn on the dispenser
            "On fertiliser dispensing".to_string()
        } else {
            // If soil nutrient level is low but fertilizer dispenser is already on
            // maintain current state
            "Maintain".to_string()
        }
    }
}

// Function to determine the action for controlling pesticide dispenser based on thermal data
fn thermal_logic(thermal: i32) -> String {
    // Lock the mutex to access the operating status of the pesticide dispenser
    let mut operating = sprayers::PESTICIDE_DIS_OPERATING.lock().unwrap();

    // Determine the action based on the thermal data
    if thermal < 31 {
        if *operating {
            // If temperature is normal and pesticide dispenser is on, turn off the dispenser
            "Off pesticide dispensing".to_string()
        } else {
            // If temperature is normal but pesticide dispenser is already off
            // maintain current state
            "Maintain".to_string()
        }
    } else {
        if !*operating {
            // If temperature is high and pesticide dispenser is off, turn on the dispenser
            "On pesticide dispensing".to_string()
        } else {
            // If temperature is high but pesticide dispenser is already on
            // maintain current state
            "Maintain".to_string()
        }
    }
}

// Function to determine the action based on GPS coordinates
fn gps_logic(longitude: i32, latitude: i32) -> String {
    // Preset path coordinates
    let p_longitude_start_end = 5;
    let p_latitude_start = 100;
    let p_latitude_end = 130;

    // Lock the mutex to access and update the latest latitude
    {
        let mut latest_latitude = LATEST_LATITUDE.lock().unwrap();
        if latitude > *latest_latitude {
            *latest_latitude = latitude;
        }
    }
    // Lock the mutex to access and update the resume operation flag
    {
        let mut resume = RESUME_OPERATION.lock().unwrap();
        let mut latest_latitude = LATEST_LATITUDE.lock().unwrap();
        if !*resume {
            // Set resume operation to true if reached last operate location
            if latitude >= *latest_latitude {
                *resume = true;
                println!("Drone: Reached last location [{}, {}], operation rusumed....",
                         longitude, latitude);
            } }
    }

    // Lock the mutexes to access the task ongoing, back home, and landing flags
    let mut task_ongoing = TASK_ONGOING.lock().unwrap();
    let mut back_home = BACK_HOME.lock().unwrap();
    let mut landing = LANDING.lock().unwrap();

    // If drone reached the preset end GPS coordinates, then back home
    if *task_ongoing && !*back_home && latitude == p_latitude_end {
        *task_ongoing = false;
        *back_home = true;
        println!("Drone mission accomplished! Backing to home...");
    } else if *back_home && latitude == p_latitude_start {
        // If drone reached is backing home and reached the start point, set landing to true
        if !*landing {
            *landing = true;
        }
    }

    // Calculate the difference between preset and current longitude and return as a string
    let difference = p_longitude_start_end - longitude;
    difference.to_string()
}

// Function to determine the action based on obstacles detection data
fn obst_dect_logic(obstacles_dtc: i32) -> String {
    // If obstacles are detected within 100 meters, increase thrust to avoid obstacles
    if obstacles_dtc < 100 {
        "Increase thrust to avoid obstacles".to_string()
    } else {
        // If no obstacles are detected within 100 meters, maintain the current thrust
        "Maintain thrust".to_string()
    }
}

// Function to determine the action for gear deployment based on the altitude
fn altitude_gear_logic(altitude: i32) -> String {
    // Lock the mutex to access the gear status
    let gear_up = gear::GEARUP.lock().unwrap();

    // Lock the mutex to access the landing status
    let mut landing = LANDING.lock().unwrap();

    // Determine the action based on the altitude and gear status
    if !*gear_up && altitude > 5 {
        // If the gear is not up and altitude is greater than 5 meters, retract the gear
        "Retract gear".to_string()
    } else if *gear_up && altitude < 20 && *landing {
        // If the gear is up, altitude is less than 20 meters, and landing is in progress,
        // deploy the gear
        "Deploy gear".to_string()
    } else {
        // Otherwise, maintain the current state of the gear
        "Maintain".to_string()
    }
}

// Function to determine the action for motor thrust based on the altitude
fn altitude_motor_logic(altitude: i32) -> String {
    // Lock the mutex to access the landing status
    let mut landing = LANDING.lock().unwrap();

    // Determine the action based on the altitude and landing status
    if !*landing && altitude < 50 {
        // If landing is not in progress and altitude is less than 50 meters
        // increase thrust for more lift
        "Increase thrust for more lift".to_string()
    } else if !*landing && altitude > 50 {
        // If landing is not in progress and altitude is greater than 50 meters
        // reduce thrust to lower altitude
        "Reduce thrust to lower".to_string()
    } else if *landing {
        // If landing is in progress, reduce thrust for landing
        "Reduce thrust for landing".to_string()
    } else {
        // Otherwise, maintain the current thrust
        "Maintain thrust".to_string()
    }
}

// Function to determine the action based on battery level
fn battery_logic(batt: i32) {
    // Lock the mutexes to access the back home and resume operation flags
    let mut back_home = BACK_HOME.lock().unwrap();
    let mut resume = RESUME_OPERATION.lock().unwrap();

    // If battery level is below 21% and the drone is not already heading back home
    if batt < 21 && !*back_home {
        println!("Battery: Low battery, going home....");

        // Set flags to return home and suspend sprayers operations
        *back_home = true;
        *resume = false;
        operation_suspend();
    }
}

// Function to determine the action based on water tank level
fn water_lvl_logic(water_lvl: i32) {
    // Lock the mutexes to access the back home and resume operation flags
    let mut back_home = BACK_HOME.lock().unwrap();
    let mut resume = RESUME_OPERATION.lock().unwrap();

    // If water tank level is empty and the drone is not already heading back home
    if water_lvl <= 0 && !*back_home {
        println!("Water Tank: Empty, going home....");

        // Set flags to return home and suspend sprayers operations
        *back_home = true;
        *resume = false;
        operation_suspend();
    }
}

// Function to determine the action based on fertilizer tank level
fn fert_lvl_logic(fert_lvl: i32) {
    // Lock the mutexes to access the back home and resume operation flags
    let mut back_home = BACK_HOME.lock().unwrap();
    let mut resume = RESUME_OPERATION.lock().unwrap();

    // If fertilizer tank level is empty and the drone is not already heading back home
    if fert_lvl <= 0 && !*back_home {
        println!("Fertiliser Tank: Empty, going home....");

        // Set flags to return home and suspend sprayers operations
        *back_home = true;
        *resume = false;
        operation_suspend();
    }
}

// Function to determine the action based on pesticide tank level
fn pest_lvl_logic(pest_lvl: i32) {
    // Lock the mutexes to access the back home and resume operation flags
    let mut back_home = BACK_HOME.lock().unwrap();
    let mut resume = RESUME_OPERATION.lock().unwrap();

    // If pesticide tank level is empty and the drone is not already heading back home
    if pest_lvl <= 0 && !*back_home {
        println!("Pesticide Tank: Empty, going home....");

        // Set flags to return home and suspend sprayers operations
        *back_home = true;
        *resume = false;
        operation_suspend();
    }
}