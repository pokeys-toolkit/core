//! Configuration Loader Example
//!
//! This example shows how to load a YAML configuration file and apply it
//! to a physical PoKeys device, demonstrating the complete workflow from
//! configuration file to working hardware.

use pokeys_config::*;
use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Configuration Loader Example");
    println!("===============================");
    println!("This example loads a YAML configuration and applies it to a physical device.\n");

    // Step 1: Load configuration from YAML file
    println!("📄 Step 1: Loading Configuration from YAML");
    println!("===========================================");

    let config_path = "examples/config/step_by_step_config.yaml";
    println!("Loading configuration from: {}", config_path);

    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file '{}': {}", config_path, e))?;

    let config: SystemConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    println!("✅ Configuration loaded successfully");

    // Get the device configuration
    let device_config = config
        .devices
        .get("demo_device")
        .ok_or("Device 'demo_device' not found in configuration")?;

    println!("   Device: {}", device_config.name);
    println!(
        "   Description: {}",
        device_config
            .description
            .as_ref()
            .unwrap_or(&"None".to_string())
    );
    println!("   Target Serial: {:?}", device_config.serial_number);

    // Step 2: Validate configuration
    println!("\n🔍 Step 2: Validating Configuration");
    println!("===================================");

    device_config
        .validate_device_config()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    println!("✅ Configuration validation passed");

    // Show configuration summary
    show_config_summary(device_config);

    // Step 3: Connect to physical device
    println!("\n🔌 Step 3: Connecting to Physical Device");
    println!("========================================");

    let target_serial = device_config.serial_number.unwrap_or(32218);
    let mut device = match connect_to_device_with_serial(target_serial, true, 3000) {
        Ok(device) => {
            println!("✅ Connected to target device (Serial: {})", target_serial);
            device
        }
        Err(_) => {
            println!(
                "⚠️  Target device {} not found, trying any available device...",
                target_serial
            );

            let device_count = enumerate_usb_devices()?;
            if device_count > 0 {
                let device = connect_to_device(0)?;
                println!("✅ Connected to available device (Index: 0)");
                device
            } else {
                return Err(
                    "No PoKeys devices found! Please connect a device and try again.".into(),
                );
            }
        }
    };

    // Get device info
    let device_info = device.get_device_info()?;
    println!("   Device: {}", device_info.device_name);
    println!(
        "   Firmware: {}.{}",
        device_info.firmware_version_major, device_info.firmware_version_minor
    );

    // Step 4: Apply configuration to device
    println!("\n⚙️  Step 4: Applying Configuration to Device");
    println!("============================================");

    apply_configuration_to_device(&mut device, device_config)?;

    println!("✅ Configuration applied successfully");

    // Step 5: Test the configured device
    println!("\n🎮 Step 5: Testing Configured Device");
    println!("====================================");

    test_configured_device(&mut device, device_config)?;

    // Step 6: Cleanup
    println!("\n🧹 Step 6: Cleanup");
    println!("==================");

    cleanup_device(&mut device, device_config)?;

    println!("✅ Device cleaned up successfully");

    println!("\n🎉 Configuration Loader Example Complete!");
    println!("=========================================");
    println!("✅ Configuration loaded from YAML file");
    println!("✅ Configuration validated successfully");
    println!("✅ Physical device configured to match YAML");
    println!("✅ Device functionality tested");
    println!("✅ Device cleaned up");

    Ok(())
}

fn show_config_summary(config: &DeviceConfig) {
    println!("\n📋 Configuration Summary");
    println!("========================");

    // Show pin configuration
    println!("🔌 Pin Configuration:");
    for (pin_num, pin_config) in &config.pins {
        if pin_config.enabled {
            println!(
                "   Pin {}: {} ({:?})",
                pin_num, pin_config.name, pin_config.function
            );
        }
    }

    // Show SPI configuration
    if let Some(ref spi_config) = config.spi {
        if spi_config.enabled {
            println!("\n🔧 SPI Configuration:");
            println!("   Name: {}", spi_config.name);
            println!("   Prescaler: 0x{:02X}", spi_config.prescaler);
            println!("   Mode: 0x{:02X}", spi_config.mode);

            let reserved_pins = config.get_spi_reserved_pins();
            println!("   Reserved pins: {:?}", reserved_pins);
        }
    }

    // Show PWM configuration
    if !config.pwm.is_empty() {
        println!("\n⚡ PWM Configuration:");
        for (name, pwm_config) in &config.pwm {
            if pwm_config.enabled {
                println!(
                    "   {}: Pin {} at {}Hz (initial: {}%)",
                    name, pwm_config.pin, pwm_config.frequency_hz, pwm_config.initial_duty_cycle
                );
            }
        }
    }

    // Show display configuration
    if !config.max7219_displays.is_empty() {
        println!("\n📺 MAX7219 Display Configuration:");
        for (name, display_config) in &config.max7219_displays {
            if display_config.enabled {
                println!(
                    "   {}: CS pin {}, {} digits, intensity {}",
                    name, display_config.cs_pin, display_config.digits, display_config.intensity
                );
            }
        }
    }
}

fn apply_configuration_to_device(
    device: &mut PoKeysDevice,
    config: &DeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Configuring pins...");

    // Configure all pins
    for (pin_num, pin_config) in &config.pins {
        if pin_config.enabled {
            let device_pin_function = match pin_config.function {
                pokeys_config::PinFunction::DigitalInput => {
                    pokeys_lib::io::PinFunction::DigitalInput
                }
                pokeys_config::PinFunction::DigitalOutput => {
                    pokeys_lib::io::PinFunction::DigitalOutput
                }
                pokeys_config::PinFunction::AnalogInput => pokeys_lib::io::PinFunction::AnalogInput,
                _ => pokeys_lib::io::PinFunction::DigitalInput, // Default
            };

            device.set_pin_function(*pin_num as u32, device_pin_function)?;

            // Set initial states for digital outputs
            if matches!(
                pin_config.function,
                pokeys_config::PinFunction::DigitalOutput
            ) {
                device.set_digital_output(*pin_num as u32, pin_config.initial_state)?;
            }

            println!("   ✅ Pin {}: {} configured", pin_num, pin_config.name);
        }
    }

    // Configure SPI
    if let Some(ref spi_config) = config.spi {
        if spi_config.enabled {
            println!("🔧 Configuring SPI...");
            device.spi_configure(spi_config.prescaler, spi_config.mode)?;
            println!("   ✅ SPI configured");
        }
    }

    // Configure PWM
    for (name, pwm_config) in &config.pwm {
        if pwm_config.enabled {
            println!("🔧 Configuring PWM: {}...", name);
            device.set_pwm_duty_cycle(0, pwm_config.initial_duty_cycle)?; // Use channel 0
            println!("   ✅ PWM {} configured", name);
        }
    }

    // Configure MAX7219 displays
    if !config.max7219_displays.is_empty() {
        println!("🔧 Configuring MAX7219 displays...");

        use pokeys_lib::devices::spi::Max7219;

        for (name, display_config) in &config.max7219_displays {
            if display_config.enabled {
                let mut display = Max7219::new(device, display_config.cs_pin)?;

                if display_config.raw_segments {
                    display.configure_raw_segments(display_config.digits)?;
                } else {
                    display.configure_numeric_display(display_config.digits)?;
                }

                display.set_intensity(display_config.intensity)?;

                // Set initial display content
                if let Some(ref initial_text) = display_config.initial_text {
                    display.display_text(initial_text)?;
                } else if let Some(initial_number) = display_config.initial_number {
                    display.display_number(initial_number)?;
                }

                println!("   ✅ Display {} configured", name);
            }
        }
    }

    Ok(())
}

fn test_configured_device(
    device: &mut PoKeysDevice,
    config: &DeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Running device tests for 5 seconds...");
    println!("   (Press buttons to see LED responses)");

    use pokeys_lib::devices::spi::Max7219;

    // Set up display if available
    let mut display = if let Some(display_config) = config.max7219_displays.values().next() {
        if display_config.enabled {
            Some(Max7219::new(device, display_config.cs_pin)?)
        } else {
            None
        }
    } else {
        None
    };

    let start_time = std::time::Instant::now();
    let mut counter = 0;

    while start_time.elapsed() < Duration::from_secs(5) {
        // Read button states (if configured)
        let start_button = if config.pins.contains_key(&1) {
            device.get_digital_input(1).unwrap_or(false)
        } else {
            false
        };

        let stop_button = if config.pins.contains_key(&2) {
            device.get_digital_input(2).unwrap_or(false)
        } else {
            false
        };

        // Control LEDs based on buttons (if configured)
        if config.pins.contains_key(&3) && config.pins.contains_key(&4) {
            if start_button {
                device.set_digital_output(3, true)?; // Green on
                device.set_digital_output(4, false)?; // Red off
                if let Some(ref mut disp) = display {
                    disp.display_text("START")?;
                }
            } else if stop_button {
                device.set_digital_output(3, false)?; // Green off
                device.set_digital_output(4, true)?; // Red on
                if let Some(ref mut disp) = display {
                    disp.display_text("STOP")?;
                }
            } else {
                device.set_digital_output(3, false)?; // Both off
                device.set_digital_output(4, false)?;
                if let Some(ref mut disp) = display {
                    disp.display_text("READY")?;
                }
            }
        }

        // Vary PWM if configured
        if !config.pwm.is_empty() {
            let pwm_value = ((counter as f32 * 0.1).sin() + 1.0) * 50.0; // 0-100%
            device.set_pwm_duty_cycle(0, pwm_value)?;
        }

        // Print status every second
        if counter % 10 == 0 {
            let remaining = 5 - start_time.elapsed().as_secs();
            println!(
                "   Status: Start={}, Stop={} | Time: {}s remaining",
                if start_button { "PRESSED" } else { "released" },
                if stop_button { "PRESSED" } else { "released" },
                remaining
            );
        }

        counter += 1;
        thread::sleep(Duration::from_millis(100));
    }

    println!("✅ Device test completed");

    Ok(())
}

fn cleanup_device(
    device: &mut PoKeysDevice,
    config: &DeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Turn off all digital outputs
    for (pin_num, pin_config) in &config.pins {
        if pin_config.enabled
            && matches!(
                pin_config.function,
                pokeys_config::PinFunction::DigitalOutput
            )
        {
            device.set_digital_output(*pin_num as u32, false)?;
            println!("   ✅ Pin {} ({}) turned off", pin_num, pin_config.name);
        }
    }

    // Turn off PWM
    for (name, pwm_config) in &config.pwm {
        if pwm_config.enabled {
            device.set_pwm_duty_cycle(0, 0.0)?;
            println!("   ✅ PWM {} turned off", name);
        }
    }

    // Set displays to show "DONE"
    if !config.max7219_displays.is_empty() {
        use pokeys_lib::devices::spi::Max7219;

        for (name, display_config) in &config.max7219_displays {
            if display_config.enabled {
                let mut display = Max7219::new(device, display_config.cs_pin)?;
                display.display_text("DONE")?;
                println!("   ✅ Display {} shows DONE", name);
            }
        }
    }

    Ok(())
}
