use std::thread;

mod sensor;
mod main_control;
mod rabbitmq_functions;
mod structs;
mod actuator;

fn main() {
    // Start the sensor in a separate thread
    let sensors_handle = thread::spawn(|| {
        sensor::sensors::generate_sensor_data();
    });

    // Start the main control in a separate thread
    let main_control_handle = thread::spawn(|| {
        main_control::main_control::data_receiver();
    });

    // Start the actuators in a separate thread
    let actuators_handle = thread::spawn(|| {
        actuator::actuators::execute_actuators();
    });

    let _ = sensors_handle.join();
    let _ = main_control_handle.join();
    let _ = actuators_handle.join();
}
