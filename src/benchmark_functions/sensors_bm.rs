use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::time::Duration;
use scheduled_thread_pool::ScheduledThreadPool;
use once_cell::sync::Lazy;
use crate::main_control::main_control::{RESUME_OPERATION, TASK_ONGOING};
use crate::main_control::main_control::BACK_HOME;
use crate::sensor::battery_management::generate_battery_level;
use crate::sensor::gps::generate_gps_data;
use crate::sensor::monitor::monitor;
use crate::sensor::multispectral_camera::generate_multispecrtal_data;
use crate::sensor::thermal::generate_thermal_data;
use crate::sensor::ultrasonic_sensor::generate_ultrasonic_data;

pub static SENSOR_OPERATING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

pub fn generate_sensor_data() {
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


    loop {
        // Send receiver to monitor
        monitor(&mts_cam_recv, &thermal_recv, &gps_recv, &ultra_sensor_recv, &batt_recv);

        // Check if sensors should be shut down
        if !SENSOR_OPERATING.load(Ordering::Relaxed) {
            println!("Sensors are shut down.");
            break;
        }
    };
}