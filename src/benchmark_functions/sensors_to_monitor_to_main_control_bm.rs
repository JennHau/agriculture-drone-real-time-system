use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::time::Duration;
use scheduled_thread_pool::ScheduledThreadPool;
use once_cell::sync::Lazy;
use crate::main_control::main_control::{MAIN_CONTROL_OPERATING, RESUME_OPERATION, TASK_ONGOING};
use crate::main_control::main_control::BACK_HOME;
use crate::benchmark_functions::battery_management_bm::generate_battery_level;
use crate::benchmark_functions::gps_bm::generate_gps_data;
use crate::benchmark_functions::monitor_bm::{monitor};
use crate::benchmark_functions::multispectral_camera_bm::generate_multispecrtal_data;
use crate::benchmark_functions::thermal_bm::generate_thermal_data;
use crate::benchmark_functions::ultrasonic_sensor_bm::generate_ultrasonic_data;
use crate::rabbitmq_functions::receive;
use crate::structs::{GPS, Mts_Camera, Ultrasonic};

pub static SENSOR_OPERATING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

pub fn bm_sensors_to_monitor_to_main_control() {
    let(mts_cam_sender, mts_cam_recv) = channel();
    let(thermal_sender, thermal_recv) = channel();
    let(gps_sender, gps_recv) = channel();
    let(ultra_sensor_sender, ultra_sensor_recv) = channel();
    let(batt_sender, batt_recv) = channel();

    let pool = ScheduledThreadPool::new(5);

    // Multispectral camera thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Acquire necessary locks and check conditions
            let back_home = BACK_HOME.lock().unwrap();
            let resume = RESUME_OPERATION.lock().unwrap();
            let task_ongoing = TASK_ONGOING.lock().unwrap();

            // Generate multispectral data if conditions are met
            if !*back_home && *resume && *task_ongoing {
                generate_multispecrtal_data(&mts_cam_sender);
            }
        }
    );

    // Thermal thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Acquire necessary locks and check conditions
            let back_home = BACK_HOME.lock().unwrap();
            let resume = RESUME_OPERATION.lock().unwrap();
            let task_ongoing = TASK_ONGOING.lock().unwrap();

            // Generate thermal data if conditions are met
            if !*back_home && *resume && *task_ongoing {
                generate_thermal_data(&thermal_sender);
            }
        }
    );

    // GPS thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Generate GPS data if conditions are met
            generate_gps_data(&gps_sender);
        }
    );

    // Ultrasonic sensor thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Generate ultrasonic sensor data if conditions are met
            generate_ultrasonic_data(&ultra_sensor_sender);
        }
    );

    // Battery thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Generate battery level if conditions are met
            generate_battery_level(&batt_sender);
        }
    );

    monitor(&mts_cam_recv, &thermal_recv, &gps_recv, &ultra_sensor_recv, &batt_recv);
    main_control_data_receiver();
}

pub fn main_control_data_receiver() {
    // Create a thread pool with 8 threads
    let pool = ScheduledThreadPool::new(5);

    // Multispectral camera thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let data = receive("mts_cam");
        }
    );

    // Thermal thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let thermal = receive("thermal");
        }
    );


    // GPS thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let data = receive("gps");
        }
    );

    // Ultrasonic sensor thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution
        move ||{
            // Receive data from the sensor
            let data = receive("ultra_sensor");
        }
    );

    // Battery thread
    pool.execute_at_fixed_rate(
        Duration::from_secs(0), // initial delay
        Duration::from_secs(1), // rate of execution\
        move ||{
            // Receive data from the sensor
            let batt = receive("batt");
        }
    );
}

