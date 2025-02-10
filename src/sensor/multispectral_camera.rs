use std::sync::mpsc::Sender;
use std::sync::Mutex;
use rand::Rng;
use crate::structs::Mts_Camera;

static DRY_MOISTURE: Mutex<bool> = Mutex::new(false);
static SOIL_MOIST_VALUE: Mutex<i32> = Mutex::new(0);
static INS_NUTR: Mutex<bool> = Mutex::new(false);
static NUTR_VALUE: Mutex<f64> = Mutex::new(0.0);


// Function to generate multispectral data and send it via the provided channel
pub fn generate_multispecrtal_data(mts_cam_sender: &Sender<Mts_Camera>) {
    // Lock the moisture state variables
    let mut dry_moisture = DRY_MOISTURE.lock().unwrap();
    let mut soil_moist_value = SOIL_MOIST_VALUE.lock().unwrap();

    // Create a random number generator
    let mut rng = rand::thread_rng();
    // Generate a random number between 0 and 9
    let prob = rng.gen_range(0..10); // Generate a random number between 0 and 9

    // Determine soil moisture value based on probability and current moisture state
    if prob < 7 && !*dry_moisture {
        // 70% probability for moderate soil moisture
        *soil_moist_value = rng.gen_range(300..401); // Moderate soil moisture range

    } else if prob > 6 && !*dry_moisture {
        // 30% probability for dry soil moisture
        *soil_moist_value = rng.gen_range(200..300); // Dry soil moisture range
        *dry_moisture = true;

    } else if *dry_moisture {
        // If soil was previously dry, gradually increase moisture
        *soil_moist_value += 20;
        if *soil_moist_value > 299 {
            *dry_moisture = false;
        }
    };


    // Lock the nutrient state variables
    let mut ins_nutr = INS_NUTR.lock().unwrap();
    let mut nutr_value = NUTR_VALUE.lock().unwrap();

    // Determine nutrient value based on probability and current nutrient state
    if prob < 7 && !*ins_nutr {
        // 70% probability for adequate nutrient value
        let rd_nutr_value: f64 = rng.gen_range(0.6..1.0); // Adequate nutrient range
        let rounded_nutr_value = (rd_nutr_value * 10.0).round() / 10.0;
        *nutr_value = rounded_nutr_value;

    } else if prob > 6 && !*ins_nutr {
        // 30% probability for low nutrient value
        let rd_nutr_value: f64  = rng.gen_range(0.1..0.6); // Low nutrient range
        let rounded_nutr_value = (rd_nutr_value * 10.0).round() / 10.0;
        *nutr_value = rounded_nutr_value;
        *ins_nutr = true;

    } else if *ins_nutr {
        // If nutrient was previously low, gradually increase nutrient level
        *nutr_value += 0.1;
        if *nutr_value > 0.5 {
            *ins_nutr = false;
        }
    };

    // Prepare multispectral data struct with current soil moisture and nutrient values
    let soil_moist = *soil_moist_value;
    let soil_nutr = *nutr_value;

    let mts_data = Mts_Camera {
        soil_moist,
        soil_nutr
    };

    // Send the generated multispectral data through the provided sender channel
    mts_cam_sender.send(mts_data).unwrap();
}

