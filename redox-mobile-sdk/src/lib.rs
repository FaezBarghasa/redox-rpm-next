//! # Redox Mobile SDK
//!
//! Software Development Kit providing high-level, safe Rust APIs for mobile
//! application development on Redox Mobile OS.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSensorReading {
    pub sensor_type: String,
    pub values: Vec<f32>,
    pub timestamp_ns: u64,
}

pub struct SensorManager;

impl SensorManager {
    pub fn new() -> Self {
        Self
    }

    pub fn read_accelerometer(&self) -> Result<MobileSensorReading, String> {
        Ok(MobileSensorReading {
            sensor_type: "Accelerometer".into(),
            values: vec![0.0, 9.81, 0.0],
            timestamp_ns: 1000000,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub percentage: u8,
    pub is_charging: bool,
    pub temperature_celsius: f32,
}

pub struct PowerManager;

impl PowerManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_battery_state(&self) -> Result<BatteryState, String> {
        Ok(BatteryState {
            percentage: 95,
            is_charging: false,
            temperature_celsius: 30.5,
        })
    }

    pub fn acquire_wake_lock(&self, tag: &str) -> Result<u64, String> {
        println!("[mobile-sdk] Acquired wake lock for tag: {}", tag);
        Ok(1)
    }
}

pub struct TelephonyManager;

impl TelephonyManager {
    pub fn new() -> Self {
        Self
    }

    pub fn dial_number(&self, phone_number: &str) -> Result<u32, String> {
        println!("[mobile-sdk] Dialing number: {}", phone_number);
        Ok(101)
    }

    pub fn send_sms(&self, recipient: &str, body: &str) -> Result<u32, String> {
        println!("[mobile-sdk] Sending SMS to {}: {}", recipient, body);
        Ok(201)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_sdk_apis() {
        let sensor_mgr = SensorManager::new();
        let accel = sensor_mgr.read_accelerometer().expect("Accel read failed");
        assert_eq!(accel.sensor_type, "Accelerometer");

        let power_mgr = PowerManager::new();
        let bat = power_mgr.get_battery_state().expect("Battery read failed");
        assert_eq!(bat.percentage, 95);

        let telephony_mgr = TelephonyManager::new();
        let call_id = telephony_mgr.dial_number("+15550199").unwrap();
        assert_eq!(call_id, 101);
    }
}
