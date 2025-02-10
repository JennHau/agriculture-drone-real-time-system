use std::sync::mpsc::{channel, Receiver};
use rand::Rng;

pub fn motor_gps(instruc_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match instruc_r.try_recv() {
        Ok(instruc_1) => {
            // Create a new channel for communication with propellers
            let(i1_s, i1_r) = channel();

            // Initialize random number generator
            let mut rng = rand::thread_rng();
            // Handle different GPS instructions
            if instruc_1 == "0" {
                // If no deviation from original path
                println!("Motor: Thrust remained at 80% (on-track)");
                i1_s.send("Remain".to_string()).unwrap();

            } else {
                // If there is deviation from original path
                // Generate a random thrust value between 81 and 100
                let thrust = rng.gen_range(81..101);
                println!("Motor: Thrust increased to {}% (off-track)", thrust);

                // Convert string to integer
                match instruc_1.parse::<i32>() {
                    Ok(temp) =>
                        if temp < 0 {
                            // Roll left by the value
                            let instruc = format!("Roll left {} degree", temp * -1);
                            i1_s.send(instruc).unwrap();

                        } else {
                            // Roll right by the value
                            let instruc = format!("Roll right {} degree", temp);
                            i1_s.send(instruc).unwrap();
                        } Err(_) => () }}
            // Pass the instruction to the propellers
            propeller_gps(&i1_r);
        } Err(_) => {}
    }
}

pub fn motor_obs_dtc(instruc_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match instruc_r.try_recv() {
        Ok(instruc_2) => {
            // Create a new channel for communication with propellers
            let(i2_s, i2_r) = channel();

            // Initialize random number generator
            let mut rng = rand::thread_rng();

            // Handle obstacle detection instructions
            if instruc_2 == "Increase thrust to avoid obstacles" {
                // Generate a random thrust value between 81 and 100
                let thrust = rng.gen_range(81..101);
                println!("Motor: Thrust increased to {}% (obstacles detected)", thrust);

                // Randomly decide the direction to roll
                let direction = rng.gen_range(0..2);
                if direction == 0 {
                    i2_s.send("Roll to left".to_string()).unwrap();

                } else {
                    i2_s.send("Roll to right".to_string()).unwrap();
                }

            } else if instruc_2 == "Maintain thrust" {
                println!("Motor: Thrust remained at 80% (no obstacles)");
                i2_s.send("Remain".to_string()).unwrap();
            }

            // Pass the instruction to the propellers
            propeller_obs_dtc(&i2_r);
        }
        Err(_) => {}
    }
}

pub fn motor_altitude(instruc_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match instruc_r.try_recv() {
        Ok(instruc_3) => {
            // Create a new channel for communication with propellers
            let(i3_s, i3_r) = channel();

            // Initialize random number generator
            let mut rng = rand::thread_rng();

            // Handle altitude control instructions
            if instruc_3 == "Increase thrust for more lift" {
                // Generate a random thrust value between 81 and 100
                let thrust = rng.gen_range(81..101);
                println!("Motor: Thrust increased to {}% (altitude low)", thrust);
                i3_s.send("Lifting the drone".to_string()).unwrap();

            } else if instruc_3 == "Reduce thrust to lower" {
                // Generate a random thrust value between 50 and 79
                let thrust = rng.gen_range(50..80);
                println!("Motor: Thrust reduced to {}% (altitude high)", thrust);
                i3_s.send("Lowering the drone (altitude high)".to_string()).unwrap();

            } else if instruc_3 == "Maintain thrust" {
                println!("Motor: Thrust remained at 80% (normal altitude)");
                i3_s.send("Remain".to_string()).unwrap();

            } else if instruc_3 == "Reduce thrust for landing" {
                println!("Motor: Reducing trust (landing)");
                i3_s.send("Lowering the drone (landing)".to_string()).unwrap();
            }
            // Pass the instruction to the propellers
            propeller_altiude(&i3_r);
        }
        Err(_) => {}
    }
}

fn propeller_gps(i1_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match i1_r.try_recv() {
        Ok(i1) => {
            // Handle instructions for propellers based on GPS data
            if i1 == "Remain" {
                println!("Propellers: Remain spinning degree (on track)");
            } else {
                println!("Propellers: {} (off track)", i1);
            }
        }
        Err(_) => {}
    }
}

fn propeller_obs_dtc(i2_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match i2_r.try_recv() {
        Ok(i2) => {
            // Handle instructions for propellers based on obstacle detection data
            if i2 == "Remain" {
                println!("Propellers: Remain spinning degree (no obstacles)");
            } else {
                println!("Propellers: {} (obstacles detected)", i2);
            }
        }
        Err(_) => {}
    }
}

fn propeller_altiude(i3_r: &Receiver<String>) {
    // Attempt to receive an instruction from the channel
    match i3_r.try_recv() {
        Ok(i3) => {
            // Handle instructions for propellers based on altitude data
            if i3 == "Remain" {
                println!("Propellers: Remain spinning degree (normal altitude)");
            } else if i3 == "Lifting the drone" {
                println!("Propellers: {} (altitude low)", i3);
            } else {
                println!("Propellers: {}", i3);
            }
        }
        Err(_) => {}
    }
}