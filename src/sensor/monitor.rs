use std::sync::mpsc::Receiver;
use crate::rabbitmq_functions::send;
use crate::structs::{GPS, Mts_Camera, Ultrasonic};

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


