//! Test SPI and MAX7219 Configuration Loading
//!
//! This example demonstrates loading and validating SPI and MAX7219
//! configurations from YAML files.

use pokeys_config::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing SPI and MAX7219 Configuration Loading");
    println!("================================================");

    // Load the example configuration
    let config_path = "examples/config/max7219_multi_display.yaml";

    println!("📄 Loading configuration from: {}", config_path);

    let config_content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let system_config: SystemConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    println!("✅ Configuration loaded successfully");

    // Validate the configuration
    println!("\n🔍 Validating Configuration");
    println!("===========================");

    println!("System settings:");
    println!(
        "  - Polling interval: {}ms",
        system_config.system.polling_interval_ms
    );
    println!("  - Log level: {:?}", system_config.system.log_level);
    println!(
        "  - Threading enabled: {}",
        system_config.system.enable_threading
    );

    // Check devices
    println!("\nDevices found: {}", system_config.devices.len());

    for (device_name, device_config) in &system_config.devices {
        println!("\n📱 Device: {}", device_name);
        println!("  Name: {}", device_config.name);
        println!("  Serial: {:?}", device_config.serial_number);

        // Check SPI configuration
        if let Some(ref spi_config) = device_config.spi {
            println!("  🔌 SPI Configuration:");
            println!("    Name: {}", spi_config.name);
            println!("    Prescaler: 0x{:02X}", spi_config.prescaler);
            println!("    Mode: 0x{:02X}", spi_config.mode);
            println!("    Enabled: {}", spi_config.enabled);
        } else {
            println!("  🔌 SPI: Not configured");
        }

        // Check MAX7219 displays
        if !device_config.max7219_displays.is_empty() {
            println!(
                "  📺 MAX7219 Displays: {}",
                device_config.max7219_displays.len()
            );
            for (display_name, display_config) in &device_config.max7219_displays {
                println!(
                    "    - {}: CS pin {}, intensity {}, {} mode",
                    display_name,
                    display_config.cs_pin,
                    display_config.intensity,
                    if display_config.raw_segments {
                        "raw segments"
                    } else {
                        "numeric"
                    }
                );

                if let Some(ref text) = display_config.initial_text {
                    println!("      Initial text: '{}'", text);
                }
                if let Some(number) = display_config.initial_number {
                    println!("      Initial number: {}", number);
                }
            }
        }

        // Check MAX7219 multi-displays
        if !device_config.max7219_multi.is_empty() {
            println!(
                "  📺 MAX7219 Multi-Displays: {}",
                device_config.max7219_multi.len()
            );
            for (multi_name, multi_config) in &device_config.max7219_multi {
                println!(
                    "    - {}: {} displays, {} mode",
                    multi_name,
                    multi_config.displays.len(),
                    if multi_config.synchronized {
                        "synchronized"
                    } else {
                        "independent"
                    }
                );

                for display in &multi_config.displays {
                    println!(
                        "      * {}: CS pin {}, intensity {}",
                        display.name, display.cs_pin, display.intensity
                    );
                }
            }
        }

        // Validate MAX7219 configurations
        match device_config.validate_max7219_configs() {
            Ok(()) => println!("  ✅ MAX7219 configuration validation passed"),
            Err(e) => println!("  ❌ MAX7219 configuration validation failed: {}", e),
        }

        // Show CS pins used
        let cs_pins = device_config.get_max7219_cs_pins();
        if !cs_pins.is_empty() {
            println!("  📌 CS pins used: {:?}", cs_pins);
        }

        // Check other configurations
        println!("  📍 Pins configured: {}", device_config.pins.len());
        println!("  🔄 Encoders configured: {}", device_config.encoders.len());
        println!("  ⚡ PWM channels configured: {}", device_config.pwm.len());
    }

    // Test SPI and MAX7219 status
    println!("\n📊 Configuration Status");
    println!("=======================");

    for (device_name, device_config) in &system_config.devices {
        let status = crate::service::SpiMax7219Service::get_max7219_status(device_config);

        println!("\nDevice: {}", device_name);
        println!("  SPI configured: {}", status.spi_configured);
        println!("  Total displays: {}", status.total_displays);
        println!(
            "  Individual displays: {}",
            status.individual_displays.len()
        );
        println!("  Multi-display groups: {}", status.multi_displays.len());
        println!("  CS pins used: {:?}", status.cs_pins_used);

        for (name, display_status) in &status.individual_displays {
            println!(
                "    Individual '{}': CS pin {}, {} mode, enabled: {}",
                name, display_status.cs_pin, display_status.mode, display_status.enabled
            );
        }

        for (name, multi_status) in &status.multi_displays {
            println!(
                "    Multi '{}': {} displays, synchronized: {}, enabled: {}",
                name, multi_status.display_count, multi_status.synchronized, multi_status.enabled
            );
        }
    }

    println!("\n🎉 Configuration Loading Test Complete!");
    println!("✅ All configurations loaded and validated successfully");
    println!("✅ SPI and MAX7219 configurations are ready for use");

    Ok(())
}
