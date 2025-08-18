//! SPI System Integration Test
//! 
//! This example demonstrates the complete SPI system integration including:
//! - Device model validation
//! - Configuration validation  
//! - SPI pin reservation enforcement
//! - MAX7219 multi-display support

use pokeys_lib::models::DeviceModel;
use pokeys_config::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 SPI System Integration Test");
    println!("==============================");

    // Test 1: Device Model Validation
    println!("\n📱 Test 1: Device Model Validation");
    println!("==================================");
    
    let model = DeviceModel::from_file("pokeys-lib/models/PoKeys57U.yaml")?;
    println!("✅ Device model loaded: {}", model.name);
    
    // Verify SPI capabilities
    let pin_23 = model.pins.get(&23).unwrap();
    let pin_25 = model.pins.get(&25).unwrap();
    
    assert!(pin_23.capabilities.contains(&"SpiMosi".to_string()));
    assert!(pin_25.capabilities.contains(&"SpiClock".to_string()));
    println!("✅ SPI pin capabilities verified");
    
    // Count CS-capable pins
    let cs_pins: Vec<u8> = model.pins.iter()
        .filter(|(_, pin)| pin.capabilities.contains(&"SpiChipSelect".to_string()))
        .map(|(pin_num, _)| *pin_num)
        .collect();
    
    println!("✅ CS-capable pins: {} pins available", cs_pins.len());
    assert!(cs_pins.len() >= 30, "Should have at least 30 CS-capable pins");

    // Test 2: Valid Configuration
    println!("\n📋 Test 2: Valid Configuration");
    println!("==============================");
    
    let valid_config = r#"
system:
  polling_interval_ms: 100
  log_level: info

devices:
  production_controller:
    name: "Production Line Controller"
    serial_number: 32218
    connection:
      type: usb
      usb:
        serial_number: 32218
    
    spi:
      name: "Display SPI Bus"
      enabled: true
      prescaler: 0x04
      mode: 0x00
    
    pins:
      23:
        name: "SPI MOSI"
        function: SpiMosi
        enabled: true
      25:
        name: "SPI CLK"
        function: SpiClock
        enabled: true
      1:
        name: "Start Button"
        function: DigitalInput
        enabled: true
    
    max7219_displays:
      status_display:
        name: "Status Display"
        cs_pin: 24
        intensity: 8
        enabled: true
      counter_display:
        name: "Counter Display"
        cs_pin: 26
        intensity: 10
        enabled: true
    
    max7219_multi:
      production_line:
        displays:
          - name: "Line 1"
            cs_pin: 27
          - name: "Line 2"
            cs_pin: 28
        intensity: 8
        raw_segments: true
        enabled: true
"#;

    let config: SystemConfig = serde_yaml::from_str(valid_config)?;
    let device_config = config.devices.get("production_controller").unwrap();
    
    // Validate the configuration
    device_config.validate_device_config()?;
    println!("✅ Valid configuration passed validation");
    
    // Verify SPI reserved pins
    let reserved_pins = device_config.get_spi_reserved_pins();
    assert_eq!(reserved_pins, vec![23, 25]);
    println!("✅ SPI reserved pins: {:?}", reserved_pins);

    // Test 3: Invalid Configuration - Pin Conflict
    println!("\n❌ Test 3: Invalid Configuration - Pin Conflict");
    println!("===============================================");
    
    let invalid_config = r#"
system:
  polling_interval_ms: 100
  log_level: info

devices:
  test_device:
    name: "Test Device"
    serial_number: 32218
    connection:
      type: usb
      usb:
        serial_number: 32218
    
    spi:
      name: "Main SPI"
      enabled: true
    
    pins:
      23:
        name: "LED Output"
        function: DigitalOutput
        enabled: true
    
    max7219_displays:
      display1:
        name: "Display 1"
        cs_pin: 24
        intensity: 8
        enabled: true
"#;

    let config: SystemConfig = serde_yaml::from_str(invalid_config)?;
    let device_config = config.devices.get("test_device").unwrap();
    
    match device_config.validate_device_config() {
        Ok(()) => {
            println!("❌ Invalid configuration incorrectly passed validation");
            return Err("Validation should have failed".into());
        }
        Err(e) => {
            println!("✅ Invalid configuration correctly rejected: {}", e);
            assert!(e.contains("Pin 23"));
            assert!(e.contains("DigitalOutput"));
            assert!(e.contains("SPI"));
        }
    }

    // Test 4: Invalid Configuration - CS Pin Conflict
    println!("\n❌ Test 4: Invalid Configuration - CS Pin Conflict");
    println!("==================================================");
    
    let invalid_cs_config = r#"
system:
  polling_interval_ms: 100
  log_level: info

devices:
  test_device:
    name: "Test Device"
    serial_number: 32218
    connection:
      type: usb
      usb:
        serial_number: 32218
    
    spi:
      name: "Main SPI"
      enabled: true
    
    max7219_displays:
      display1:
        name: "Display 1"
        cs_pin: 23  # This conflicts with SPI MOSI
        intensity: 8
        enabled: true
"#;

    let config: SystemConfig = serde_yaml::from_str(invalid_cs_config)?;
    let device_config = config.devices.get("test_device").unwrap();
    
    match device_config.validate_device_config() {
        Ok(()) => {
            println!("❌ Invalid CS configuration incorrectly passed validation");
            return Err("CS validation should have failed".into());
        }
        Err(e) => {
            println!("✅ Invalid CS configuration correctly rejected: {}", e);
            assert!(e.contains("pin 23"));
            assert!(e.contains("SPI MOSI"));
        }
    }

    // Test 5: SPI Disabled - Pins Available
    println!("\n✅ Test 5: SPI Disabled - Pins Available");
    println!("========================================");
    
    let spi_disabled_config = r#"
system:
  polling_interval_ms: 100
  log_level: info

devices:
  test_device:
    name: "Test Device"
    serial_number: 32218
    connection:
      type: usb
      usb:
        serial_number: 32218
    
    spi:
      name: "Main SPI"
      enabled: false
    
    pins:
      23:
        name: "LED Output"
        function: DigitalOutput
        enabled: true
      25:
        name: "PWM Output"
        function: PWM_1
        enabled: true
"#;

    let config: SystemConfig = serde_yaml::from_str(spi_disabled_config)?;
    let device_config = config.devices.get("test_device").unwrap();
    
    device_config.validate_device_config()?;
    println!("✅ SPI disabled configuration passed validation");
    
    let reserved_pins = device_config.get_spi_reserved_pins();
    assert!(reserved_pins.is_empty());
    println!("✅ No pins reserved when SPI disabled: {:?}", reserved_pins);

    // Test 6: Multi-Display Configuration
    println!("\n📺 Test 6: Multi-Display Configuration");
    println!("======================================");
    
    let multi_display_config = r#"
system:
  polling_interval_ms: 100
  log_level: info

devices:
  multi_display_controller:
    name: "Multi-Display Controller"
    serial_number: 32218
    connection:
      type: usb
      usb:
        serial_number: 32218
    
    spi:
      name: "Display SPI Bus"
      enabled: true
      prescaler: 0x04
      mode: 0x00
    
    max7219_displays:
      display_a:
        name: "Display A"
        cs_pin: 24
        intensity: 8
        enabled: true
      display_b:
        name: "Display B"
        cs_pin: 26
        intensity: 8
        enabled: true
      display_c:
        name: "Display C"
        cs_pin: 27
        intensity: 8
        enabled: true
    
    max7219_multi:
      production_group:
        displays:
          - name: "Line 1"
            cs_pin: 28
          - name: "Line 2"
            cs_pin: 29
          - name: "Line 3"
            cs_pin: 30
        intensity: 10
        raw_segments: true
        enabled: true
"#;

    let config: SystemConfig = serde_yaml::from_str(multi_display_config)?;
    let device_config = config.devices.get("multi_display_controller").unwrap();
    
    device_config.validate_device_config()?;
    println!("✅ Multi-display configuration passed validation");
    
    // Count total displays
    let single_displays = device_config.max7219_displays.len();
    let multi_displays: usize = device_config.max7219_multi.values()
        .map(|multi| multi.displays.len())
        .sum();
    let total_displays = single_displays + multi_displays;
    
    println!("✅ Total displays configured: {} (3 single + 3 multi)", total_displays);
    assert_eq!(total_displays, 6);

    // Test Summary
    println!("\n🎉 SPI System Integration Test Complete!");
    println!("========================================");
    println!("✅ Device model validation: PASSED");
    println!("✅ Valid configuration: PASSED");
    println!("✅ Pin conflict detection: PASSED");
    println!("✅ CS pin conflict detection: PASSED");
    println!("✅ SPI disabled handling: PASSED");
    println!("✅ Multi-display support: PASSED");
    println!("\n🛡️ SPI System Features Verified:");
    println!("   🔌 Pin 23 (MOSI) and Pin 25 (CLK) reservation");
    println!("   📺 31-33 CS pins available per device");
    println!("   🛡️ Configuration validation prevents conflicts");
    println!("   📋 Clear error messages for invalid configurations");
    println!("   🎯 Multi-display support with individual CS pins");
    println!("   ✅ Backward compatibility when SPI is disabled");
    
    Ok(())
}
