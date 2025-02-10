use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mts_Camera {
    pub soil_moist: i32,
    pub soil_nutr: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ultrasonic {
    pub obstacles_dtc: i32,
    pub altitude: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPS {
    pub longitude: i32,
    pub latitude: i32
}