use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use scheduled_thread_pool::ScheduledThreadPool;
use crate::benchmark_functions::sensors_to_monitor_to_main_control_bm::main_control_data_receiver;
use crate::main_control::main_control::MAIN_CONTROL_OPERATING;
use crate::rabbitmq_functions::{receive, send};
use crate::structs::{GPS, Mts_Camera, Ultrasonic};

pub fn bm_monitor() {
    let(mts_cam_sender, mts_cam_recv) = channel();
    let(thermal_sender, thermal_recv) = channel();
    let(gps_sender, gps_recv) = channel();
    let(ultra_sensor_sender, ultra_sensor_recv) = channel();
    let(batt_sender, batt_recv) = channel();


    let mts_data = Mts_Camera {
        soil_moist: 300,
        soil_nutr: 0.6
    };

    let gps = GPS {
        longitude: 5,
        latitude: 100
    };

    let ultra_data = Ultrasonic {
        obstacles_dtc: 150,
        altitude: 50
    };

    mts_cam_sender.send(mts_data).unwrap();
    thermal_sender.send(30).unwrap();
    gps_sender.send(gps).unwrap();
    ultra_sensor_sender.send(ultra_data).unwrap();
    batt_sender.send(100).unwrap();

    monitor(&mts_cam_recv, &thermal_recv, &gps_recv, &ultra_sensor_recv, &batt_recv);
}

pub fn monitor(mts_cam_recv: &Receiver<Mts_Camera>, thermal_recv: &Receiver<i32>,
               gps_recv: &Receiver<GPS>, ultra_sensor_recv: &Receiver<Ultrasonic>,
               batt_recv: &Receiver<i32>) {

    // Define queue names for each sensor type
    let mts_came_queue = "mts_cam".to_string();
    let thermal_queue = "thermal".to_string();
    let gps_queue = "gps".to_string();
    let ultra_sensor_queue = "ultra_sensor".to_string();
    let batt_queue = "batt".to_string();


    // Monitor each sensor receiver for incoming data
    // If data is received send it to the appropriate RabbitMQ queue
    match mts_cam_recv.try_recv() {
        Ok(mts_data) => {
            // serialize the data
            let _ = send(serde_json::to_string(&mts_data).unwrap(), &mts_came_queue);
        }
        Err(_) => {}
    }

    match thermal_recv.try_recv() {
        Ok(thermal) => {
            let thermal = thermal.to_string();
            let _ = send(thermal, &thermal_queue);
        }

        Err(_) => {}
    }

    match gps_recv.try_recv() {
        Ok(gps) => {
            // serialize the data
            let _ = send(serde_json::to_string(&gps).unwrap(), &gps_queue);
        }
        Err(_) => {}
    }

    match ultra_sensor_recv.try_recv() {
        Ok(ultra_data) => {
            // serialize the data
            let _ = send(serde_json::to_string(&ultra_data).unwrap(), &ultra_sensor_queue);
        }
        Err(_) => {}
    }

    match batt_recv.try_recv() {
        Ok(batt) => {
            let batt = batt.to_string();
            let _ = send(batt, &batt_queue);
        }
        Err(_) => {}
    }
}