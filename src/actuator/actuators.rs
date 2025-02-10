use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;
use once_cell::sync::Lazy;
use scheduled_thread_pool::ScheduledThreadPool;
use crate::main_control::main_control;
use crate::sensor::ultrasonic_sensor::REACHED_MAX_ALTITUDE;
use crate::main_control::main_control::BACK_HOME;
use crate::actuator::sprayers::{FERTILISER_TANK_LEVEL, PESTICIDE_TANK_LEVEL, WATER_TANK_LEVEL};
use crate::sensor::battery_management::BATTERY_LEVEL;
use crate::rabbitmq_functions::receive;
use crate::actuator::gear::gear;
use crate::actuator::motor_propellers::{motor_altitude, motor_gps, motor_obs_dtc};
use crate::actuator::sprayers;
use crate::main_control::main_control::{LANDING, TASK_ONGOING};
use crate::sensor::sensors;

pub static ACTUATOR_OPERATING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

// Function to execute various actuators
pub fn execute_actuators() {
    // Create channels for communication
    let(soil_moist_s, soil_moist_r) = channel();
    let(soil_nutr_s, soil_nutr_r) = channel();
    let(thermal_s, thermal_r) = channel();
    let(gps_s, gps_r) = channel();
    let(obs_detc_s, obs_detc_r) = channel();
    let(alt_motor_s, alt_motor_r) = channel();
    let(alt_gear_s, alt_gear_r) = channel();

    // Create a thread pool with 8 threads
    let pool = ScheduledThreadPool::new(8);

    // Water dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("soil_moist_act");
            // Send the data through the sender channel
            soil_moist_s.send(instruc).unwrap();
            // Call the actuator function
            sprayers::water_dispenser(&soil_moist_r);
        }
    );

    // Fertiliser dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("soil_nutr_act");
            // Send the data through the sender channel
            soil_nutr_s.send(instruc).unwrap();
            // Call the actuator function
            sprayers::fertiliser_dispenser(&soil_nutr_r);
        }
    );

    // Pesticide dispenser thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("thermal_act");
            // Send the data through the sender channel
            thermal_s.send(instruc).unwrap();
            // Call the actuator function
            sprayers::pesticide_dispenser(&thermal_r);
        }
    );

    // Motor thread (GPS)
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("gps_act");
            // Send the data through the sender channel
            gps_s.send(instruc).unwrap();
            // Call the actuator function
            motor_gps(&gps_r);
        }
    );

    // Motor thread (obstacle detection)
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("obs_detc_act");
            // Send the data through the sender channel
            obs_detc_s.send(instruc).unwrap();
            // Call the actuator function
            motor_obs_dtc(&obs_detc_r);
        }
    );

    // Motor thread (altitude)
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("altitude_motor_act");
            // Send the data through the sender channel
            alt_motor_s.send(instruc);
            // Call the actuator function
            motor_altitude(&alt_motor_r);
        }
    );

    // Landing gear thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let instruc = receive("altitude_gear_act");
            // Send the data through the sender channel
            alt_gear_s.send(instruc).unwrap();
            // Call the actuator function
            gear(&alt_gear_r);
        }
    );

    // Recharge and refill/ termination thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive command from the main control
            let altitude = receive("altitude_refill_act");

            // Lock the mutex to access the task ongoing and terminate status
            let mut task_ongoing = TASK_ONGOING.lock().unwrap();

            // Check if the drone has landed (altitude is "0") and the task is ongoing
            if altitude == "0" && *task_ongoing {
                // Lock the mutexes to access and update the levels of resources
                let mut water_tank = WATER_TANK_LEVEL.lock().unwrap();
                let mut fertiliser_tank = FERTILISER_TANK_LEVEL.lock().unwrap();
                let mut pesticide_tank = PESTICIDE_TANK_LEVEL.lock().unwrap();
                let mut battery_level = BATTERY_LEVEL.lock().unwrap();

                // Refill the tanks and replace the battery
                *water_tank = 100;
                *fertiliser_tank = 100;
                *pesticide_tank = 100;
                *battery_level = 100;
                println!("Drone: Landed for battery replacement and tanks refill.");
                println!("Drone: Battery is replaced and all tanks are refilled");

                {
                    // Unlock the BACK_HOME mutex and update its status
                    let mut back_home = BACK_HOME.lock().unwrap();
                    *back_home = false;
                }
                {
                    // Unlock the LANDING mutex and update its status
                    let mut landing = LANDING.lock().unwrap();
                    *landing = false;
                }
                {
                    // Unlock the REACHED_MAX_ALTITUDE mutex and update its status
                    let mut max_altitude = REACHED_MAX_ALTITUDE.lock().unwrap();
                    *max_altitude = false;
                }
            } else if altitude == "0" && !*task_ongoing {
                // If the drone has landed (altitude is "0") and the task is not ongoing
                // Shut down three main systems
                sensors::SENSOR_OPERATING.store(false, Ordering::Relaxed);
                ACTUATOR_OPERATING.store(false, Ordering::Relaxed);
                main_control::MAIN_CONTROL_OPERATING.store(false, Ordering::Relaxed);

                println!("Drone: Landed safely.");
                println!("END OF SIMULATION");

    }});

    // Main loop to keep the function running until actuators are shut down
    loop{
        if !ACTUATOR_OPERATING.load(Ordering::Relaxed) {
            println!("Actuators are shut down.");
            break;
        }
    }
}



