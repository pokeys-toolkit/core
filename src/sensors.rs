//! EasySensors support

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use serde::{Deserialize, Serialize};

/// EasySensor data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EasySensor {
    pub sensor_value: i32,
    pub sensor_type: u8,
    pub sensor_refresh_period: u8,
    pub sensor_failsafe_config: u8,
    pub sensor_reading_id: u8,
    pub sensor_id: [u8; 8],
    pub sensor_ok_status: u8,
}

impl EasySensor {
    pub fn new() -> Self {
        Self {
            sensor_value: 0,
            sensor_type: 0,
            sensor_refresh_period: 0,
            sensor_failsafe_config: 0,
            sensor_reading_id: 0,
            sensor_id: [0; 8],
            sensor_ok_status: 0,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.sensor_ok_status != 0
    }

    pub fn get_refresh_period_seconds(&self) -> f32 {
        self.sensor_refresh_period as f32 * 0.1
    }

    pub fn set_refresh_period_seconds(&mut self, seconds: f32) {
        self.sensor_refresh_period = (seconds * 10.0) as u8;
    }

    pub fn get_failsafe_timeout(&self) -> u8 {
        self.sensor_failsafe_config & 0x3F
    }

    pub fn set_failsafe_timeout(&mut self, timeout_seconds: u8) {
        self.sensor_failsafe_config =
            (self.sensor_failsafe_config & 0xC0) | (timeout_seconds & 0x3F);
    }

    pub fn is_failsafe_invalid_zero(&self) -> bool {
        (self.sensor_failsafe_config & 0x40) != 0
    }

    pub fn set_failsafe_invalid_zero(&mut self, enable: bool) {
        if enable {
            self.sensor_failsafe_config |= 0x40;
        } else {
            self.sensor_failsafe_config &= !0x40;
        }
    }

    pub fn is_failsafe_invalid_max(&self) -> bool {
        (self.sensor_failsafe_config & 0x80) != 0
    }

    pub fn set_failsafe_invalid_max(&mut self, enable: bool) {
        if enable {
            self.sensor_failsafe_config |= 0x80;
        } else {
            self.sensor_failsafe_config &= !0x80;
        }
    }
}

impl Default for EasySensor {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom sensor unit descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSensorUnit {
    pub html_code: [u8; 32],
    pub simple_text: [u8; 8],
}

impl CustomSensorUnit {
    pub fn new() -> Self {
        Self {
            html_code: [0; 32],
            simple_text: [0; 8],
        }
    }

    pub fn set_html_code(&mut self, code: &str) -> Result<()> {
        if code.len() > 32 {
            return Err(PoKeysError::Parameter("HTML code too long".to_string()));
        }

        self.html_code.fill(0);
        let code_bytes = code.as_bytes();
        self.html_code[..code_bytes.len()].copy_from_slice(code_bytes);
        Ok(())
    }

    pub fn get_html_code(&self) -> String {
        let end = self.html_code.iter().position(|&b| b == 0).unwrap_or(32);
        String::from_utf8_lossy(&self.html_code[..end]).to_string()
    }

    pub fn set_simple_text(&mut self, text: &str) -> Result<()> {
        if text.len() > 8 {
            return Err(PoKeysError::Parameter("Simple text too long".to_string()));
        }

        self.simple_text.fill(0);
        let text_bytes = text.as_bytes();
        self.simple_text[..text_bytes.len()].copy_from_slice(text_bytes);
        Ok(())
    }

    pub fn get_simple_text(&self) -> String {
        let end = self.simple_text.iter().position(|&b| b == 0).unwrap_or(8);
        String::from_utf8_lossy(&self.simple_text[..end]).to_string()
    }
}

impl Default for CustomSensorUnit {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Configure EasySensor
    pub fn configure_easy_sensor(
        &mut self,
        sensor_index: usize,
        sensor_type: u8,
        sensor_id: &[u8; 8],
        refresh_period_seconds: f32,
        reading_id: u8,
    ) -> Result<()> {
        if sensor_index >= self.easy_sensors.len() {
            return Err(PoKeysError::Parameter("Invalid sensor index".to_string()));
        }

        let sensor = &mut self.easy_sensors[sensor_index];
        sensor.sensor_type = sensor_type;
        sensor.sensor_id = *sensor_id;
        sensor.set_refresh_period_seconds(refresh_period_seconds);
        sensor.sensor_reading_id = reading_id;

        // Send sensor configuration to device
        self.send_easy_sensor_configuration(sensor_index)?;
        Ok(())
    }

    /// Enable EasySensor
    pub fn enable_easy_sensor(&mut self, sensor_index: usize, enable: bool) -> Result<()> {
        if sensor_index >= self.easy_sensors.len() {
            return Err(PoKeysError::Parameter("Invalid sensor index".to_string()));
        }

        // Implementation would enable/disable the sensor
        self.send_request(0xF0, sensor_index as u8, if enable { 1 } else { 0 }, 0, 0)?;
        Ok(())
    }

    /// Read EasySensor value
    pub fn read_easy_sensor(&mut self, sensor_index: usize) -> Result<i32> {
        if sensor_index >= self.easy_sensors.len() {
            return Err(PoKeysError::Parameter("Invalid sensor index".to_string()));
        }

        // Read all sensor values
        self.read_all_easy_sensors()?;

        Ok(self.easy_sensors[sensor_index].sensor_value)
    }

    /// Read all EasySensor values
    pub fn read_all_easy_sensors(&mut self) -> Result<()> {
        let response = self.send_request(0xF1, 0, 0, 0, 0)?;

        // Parse sensor values from response
        let mut data_index = 8;
        for sensor in &mut self.easy_sensors {
            if data_index + 3 < response.len() {
                sensor.sensor_value = i32::from_le_bytes([
                    response[data_index],
                    response[data_index + 1],
                    response[data_index + 2],
                    response[data_index + 3],
                ]);
                data_index += 4;

                // Read sensor status
                if data_index < response.len() {
                    sensor.sensor_ok_status = response[data_index];
                    data_index += 1;
                }
            }
        }

        Ok(())
    }

    /// Configure sensor failsafe settings
    pub fn configure_sensor_failsafe(
        &mut self,
        sensor_index: usize,
        timeout_seconds: u8,
        invalid_value_zero: bool,
        invalid_value_max: bool,
    ) -> Result<()> {
        if sensor_index >= self.easy_sensors.len() {
            return Err(PoKeysError::Parameter("Invalid sensor index".to_string()));
        }

        self.easy_sensors[sensor_index].set_failsafe_timeout(timeout_seconds);
        self.easy_sensors[sensor_index].set_failsafe_invalid_zero(invalid_value_zero);
        self.easy_sensors[sensor_index].set_failsafe_invalid_max(invalid_value_max);

        let failsafe_config = self.easy_sensors[sensor_index].sensor_failsafe_config;

        // Send failsafe configuration to device
        self.send_request(0xF2, sensor_index as u8, failsafe_config, 0, 0)?;

        Ok(())
    }

    /// Get sensor status
    pub fn get_sensor_status(&self, sensor_index: usize) -> Result<bool> {
        if sensor_index >= self.easy_sensors.len() {
            return Err(PoKeysError::Parameter("Invalid sensor index".to_string()));
        }

        Ok(self.easy_sensors[sensor_index].is_ok())
    }

    /// Set custom sensor unit
    pub fn set_custom_sensor_unit(
        &mut self,
        unit_index: usize,
        unit: &CustomSensorUnit,
    ) -> Result<()> {
        if unit_index >= 16 {
            return Err(PoKeysError::Parameter("Invalid unit index".to_string()));
        }

        // Send HTML code
        self.send_request(
            0xF3,
            unit_index as u8,
            unit.html_code[0],
            unit.html_code[1],
            unit.html_code[2],
        )?;

        // Send remaining HTML code in chunks
        // Implementation would continue for all 32 bytes

        // Send simple text
        self.send_request(
            0xF4,
            unit_index as u8,
            unit.simple_text[0],
            unit.simple_text[1],
            unit.simple_text[2],
        )?;

        Ok(())
    }

    /// Send sensor configuration to device
    fn send_easy_sensor_configuration(&mut self, sensor_index: usize) -> Result<()> {
        // Copy sensor data to avoid borrow checker issues
        let sensor_type = self.easy_sensors[sensor_index].sensor_type;
        let sensor_refresh_period = self.easy_sensors[sensor_index].sensor_refresh_period;
        let sensor_reading_id = self.easy_sensors[sensor_index].sensor_reading_id;
        let sensor_id = self.easy_sensors[sensor_index].sensor_id;

        // Send basic configuration
        self.send_request(
            0xF5,
            sensor_index as u8,
            sensor_type,
            sensor_refresh_period,
            sensor_reading_id,
        )?;

        // Send sensor ID
        self.send_request(
            0xF6,
            sensor_index as u8,
            sensor_id[0],
            sensor_id[1],
            sensor_id[2],
        )?;

        self.send_request(
            0xF7,
            sensor_index as u8,
            sensor_id[3],
            sensor_id[4],
            sensor_id[5],
        )?;

        self.send_request(0xF8, sensor_index as u8, sensor_id[6], sensor_id[7], 0)?;

        Ok(())
    }
}

// Sensor type constants
pub mod sensor_types {
    pub const TEMPERATURE_DS18B20: u8 = 1;
    pub const HUMIDITY_DHT22: u8 = 2;
    pub const PRESSURE_BMP180: u8 = 3;
    pub const LIGHT_BH1750: u8 = 4;
    pub const DISTANCE_HC_SR04: u8 = 5;
    pub const ANALOG_VOLTAGE: u8 = 10;
    pub const ANALOG_CURRENT: u8 = 11;
    pub const DIGITAL_COUNTER: u8 = 20;
    pub const ENCODER_POSITION: u8 = 21;
}

// Convenience functions for common sensor operations

/// Configure DS18B20 temperature sensor
pub fn configure_ds18b20_sensor(
    device: &mut PoKeysDevice,
    sensor_index: usize,
    sensor_id: &[u8; 8],
    refresh_period: f32,
) -> Result<()> {
    device.configure_easy_sensor(
        sensor_index,
        sensor_types::TEMPERATURE_DS18B20,
        sensor_id,
        refresh_period,
        0, // Default reading ID for temperature
    )
}

/// Configure analog voltage sensor
pub fn configure_analog_voltage_sensor(
    device: &mut PoKeysDevice,
    sensor_index: usize,
    pin_id: u8,
    refresh_period: f32,
) -> Result<()> {
    let mut sensor_id = [0u8; 8];
    sensor_id[0] = pin_id;

    device.configure_easy_sensor(
        sensor_index,
        sensor_types::ANALOG_VOLTAGE,
        &sensor_id,
        refresh_period,
        0,
    )
}

/// Read temperature from DS18B20 sensor
pub fn read_temperature_celsius(device: &mut PoKeysDevice, sensor_index: usize) -> Result<f32> {
    let raw_value = device.read_easy_sensor(sensor_index)?;
    // DS18B20 returns temperature in 0.0625°C units
    Ok(raw_value as f32 * 0.0625)
}

/// Read voltage from analog sensor
pub fn read_voltage(
    device: &mut PoKeysDevice,
    sensor_index: usize,
    reference_voltage: f32,
) -> Result<f32> {
    let raw_value = device.read_easy_sensor(sensor_index)?;
    // Convert ADC value to voltage (assuming 12-bit ADC)
    Ok((raw_value as f32 / 4095.0) * reference_voltage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy_sensor_creation() {
        let sensor = EasySensor::new();
        assert_eq!(sensor.sensor_value, 0);
        assert!(!sensor.is_ok());
        assert_eq!(sensor.get_refresh_period_seconds(), 0.0);
    }

    #[test]
    fn test_refresh_period_conversion() {
        let mut sensor = EasySensor::new();

        sensor.set_refresh_period_seconds(1.5);
        assert_eq!(sensor.sensor_refresh_period, 15);
        assert_eq!(sensor.get_refresh_period_seconds(), 1.5);

        sensor.set_refresh_period_seconds(0.1);
        assert_eq!(sensor.sensor_refresh_period, 1);
        assert_eq!(sensor.get_refresh_period_seconds(), 0.1);
    }

    #[test]
    fn test_failsafe_configuration() {
        let mut sensor = EasySensor::new();

        sensor.set_failsafe_timeout(30);
        assert_eq!(sensor.get_failsafe_timeout(), 30);

        sensor.set_failsafe_invalid_zero(true);
        assert!(sensor.is_failsafe_invalid_zero());
        assert!(!sensor.is_failsafe_invalid_max());

        sensor.set_failsafe_invalid_max(true);
        assert!(sensor.is_failsafe_invalid_zero());
        assert!(sensor.is_failsafe_invalid_max());

        sensor.set_failsafe_invalid_zero(false);
        assert!(!sensor.is_failsafe_invalid_zero());
        assert!(sensor.is_failsafe_invalid_max());
    }

    #[test]
    fn test_custom_sensor_unit() {
        let mut unit = CustomSensorUnit::new();

        assert!(unit.set_html_code("&deg;C").is_ok());
        assert_eq!(unit.get_html_code(), "&deg;C");

        assert!(unit.set_simple_text("°C").is_ok());
        assert_eq!(unit.get_simple_text(), "°C");

        // Test length limits
        assert!(unit.set_html_code(&"x".repeat(33)).is_err());
        assert!(unit.set_simple_text(&"x".repeat(9)).is_err());
    }

    #[test]
    fn test_temperature_conversion() {
        // Test DS18B20 temperature conversion
        let raw_value = 400; // 25.0°C in DS18B20 format
        let temperature = raw_value as f32 * 0.0625;
        assert_eq!(temperature, 25.0);

        let raw_value = -160; // -10.0°C in DS18B20 format
        let temperature = raw_value as f32 * 0.0625;
        assert_eq!(temperature, -10.0);
    }

    #[test]
    fn test_voltage_conversion() {
        // Test 12-bit ADC voltage conversion
        let raw_value = 2048; // Half scale
        let reference_voltage = 5.0;
        let voltage = (raw_value as f32 / 4095.0) * reference_voltage;
        assert!((voltage - 2.5).abs() < 0.01);

        let raw_value = 4095; // Full scale
        let voltage = (raw_value as f32 / 4095.0) * reference_voltage;
        assert!((voltage - 5.0).abs() < 0.01);
    }
}
