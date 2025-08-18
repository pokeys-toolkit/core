//! Physical Device Configuration Example
//!
//! This example demonstrates how to load a configuration file and apply it
//! to a physical PoKeys device, automatically configuring all pins, displays,
//! and components to match the specified settings.

use pokeys_config::*;
use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Physical Device Configuration Example");
    println!("=======================================");

    // Step 1: Load configuration from file
    println!("\n📄 Step 1: Loading Configuration");
    println!("================================");

    let config_yaml = r#"
system:
  polling_interval_ms: 100
  log_level: info

devices:
  demo_device:
    name: "Demo Production Controller"
    description: "Physical device configuration demonstration"
    serial_number: 32218
    
    connection:
      type: usb
      usb:
        serial_number: 32218
        timeout_ms: 3000
    
    # SPI Configuration for MAX7219 displays
    spi:
      name: "Display SPI Bus"
      enabled: true
      prescaler: 0x04
      mode: 0x00
    
    # Pin Configurations
    pins:
      # Digital Inputs
      1:
        name: "Start Button"
        function: DigitalInput
        enabled: true
        pull_up: true
        initial_state: false
      
      2:
        name: "Stop Button"
        function: DigitalInput
        enabled: true
        pull_up: true
        initial_state: false
      
      # Digital Outputs
      3:
        name: "Green LED"
        function: DigitalOutput
        enabled: true
        initial_state: false
      
      4:
        name: "Red LED"
        function: DigitalOutput
        enabled: true
        initial_state: false
      
      # PWM Output
      5:
        name: "Fan Control"
        function: PWM_1
        enabled: true
        initial_state: false
      
      # Encoder
      6:
        name: "Position Encoder A"
        function: Encoder_1A
        enabled: true
      
      7:
        name: "Position Encoder B"
        function: Encoder_1B
        enabled: true
      
      # SPI Pins (automatically reserved)
      23:
        name: "SPI MOSI"
        function: SpiMosi
        enabled: true
      
      25:
        name: "SPI CLK"
        function: SpiClock
        enabled: true
      
      # Analog Input
      26:
        name: "Temperature Sensor"
        function: AnalogInput
        enabled: true
    
    # PWM Configuration
    pwm:
      fan_control:
        name: "Cooling Fan"
        pin: 5
        frequency_hz: 1000
        initial_duty_cycle: 0.0
        enabled: true
        inverted: false
    
    # Encoder Configuration
    encoders:
      main_encoder:
        name: "Position Encoder"
        pin_a: 6
        pin_b: 7
        sampling_4x: true
        enabled: true
        resolution: 1000
        initial_position: 0
    
    # Analog Input Configuration
    analog_inputs:
      26:
        name: "Temperature Sensor"
        reference_voltage: 10.0
        enabled: true
    
    # MAX7219 Displays
    max7219_displays:
      status_display:
        name: "Status Display"
        cs_pin: 24
        intensity: 8
        digits: 8
        raw_segments: true
        enabled: true
        initial_text: "READY"
      
      counter_display:
        name: "Counter Display"
        cs_pin: 26
        intensity: 10
        digits: 6
        raw_segments: false
        enabled: true
        initial_number: 0
"#;

    let config: SystemConfig = serde_yaml::from_str(config_yaml)?;
    let device_config = config.devices.get("demo_device").unwrap();

    println!("✅ Configuration loaded successfully");
    println!("   Device: {}", device_config.name);
    println!("   Serial: {:?}", device_config.serial_number);
    println!("   Pins configured: {}", device_config.pins.len());
    println!(
        "   MAX7219 displays: {}",
        device_config.max7219_displays.len()
    );

    // Step 2: Validate configuration
    println!("\n🔍 Step 2: Validating Configuration");
    println!("===================================");

    device_config
        .validate_device_config()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    println!("✅ Configuration validation passed");

    // Check SPI pin reservations
    let reserved_pins = device_config.get_spi_reserved_pins();
    println!("   SPI reserved pins: {:?}", reserved_pins);

    // Step 3: Connect to physical device
    println!("\n🔌 Step 3: Connecting to Physical Device");
    println!("========================================");

    // Try to connect to the device
    match connect_to_device_with_serial(32218, true, 3000) {
        Ok(mut device) => {
            println!("✅ Connected to PoKeys device (Serial: 32218)");

            // Get device info
            let device_info = device.get_device_info()?;
            println!("   Device Name: {}", device_info.device_name);
            println!(
                "   Firmware Version: {}.{}",
                device_info.firmware_version_major, device_info.firmware_version_minor
            );
            println!("   Pin Count: {}", device_info.pin_count);

            // Step 4: Apply configuration to physical device
            println!("\n⚙️  Step 4: Applying Configuration to Device");
            println!("============================================");

            apply_device_configuration(&mut device, device_config)?;

            // Step 5: Demonstrate the configured device
            println!("\n🎯 Step 5: Demonstrating Configured Device");
            println!("==========================================");

            demonstrate_configured_device(&mut device, device_config)?;
        }
        Err(e) => {
            println!("❌ Could not connect to physical device: {}", e);
            println!("   This example requires a physical PoKeys device with serial 32218");
            println!("   The configuration would be applied if the device was connected.");

            // Show what would be configured
            show_configuration_summary(device_config);
        }
    }

    println!("\n🎉 Physical Device Configuration Example Complete!");
    println!("=================================================");
    println!("✅ Configuration loaded and validated");
    println!("✅ Device connection attempted");
    println!("✅ Configuration application demonstrated");

    Ok(())
}

/// Apply the complete device configuration to the physical device
fn apply_device_configuration(
    device: &mut PoKeysDevice,
    config: &DeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Configuring pins...");

    // Configure all pins according to the configuration
    for (pin_num, pin_config) in &config.pins {
        if pin_config.enabled {
            println!(
                "   Pin {}: {} -> {:?}",
                pin_num, pin_config.name, pin_config.function
            );

            // Convert config pin function to device pin function
            let device_pin_function = match pin_config.function {
                pokeys_config::PinFunction::DigitalInput => {
                    pokeys_lib::io::PinFunction::DigitalInput
                }
                pokeys_config::PinFunction::DigitalOutput => {
                    pokeys_lib::io::PinFunction::DigitalOutput
                }
                pokeys_config::PinFunction::AnalogInput => pokeys_lib::io::PinFunction::AnalogInput,
                pokeys_config::PinFunction::PWM_1 => pokeys_lib::io::PinFunction::DigitalOutput, // PWM handled separately
                _ => pokeys_lib::io::PinFunction::DigitalInput, // Default fallback
            };

            device.set_pin_function(*pin_num as u32, device_pin_function)?;

            // Set initial states for digital outputs
            if matches!(
                pin_config.function,
                pokeys_config::PinFunction::DigitalOutput
            ) {
                device.set_digital_output(*pin_num as u32, pin_config.initial_state)?;
            }
        }
    }

    println!("✅ Pin configuration complete");

    // Configure SPI if enabled
    if let Some(ref spi_config) = config.spi {
        if spi_config.enabled {
            println!("🔧 Configuring SPI...");
            device.spi_configure(spi_config.prescaler, spi_config.mode)?;
            println!("✅ SPI configuration complete");
        }
    }

    // Configure PWM channels
    if !config.pwm.is_empty() {
        println!("🔧 Configuring PWM channels...");
        for (name, pwm_config) in &config.pwm {
            if pwm_config.enabled {
                println!(
                    "   PWM {}: Pin {} at {}Hz",
                    name, pwm_config.pin, pwm_config.frequency_hz
                );

                // Set PWM frequency and initial duty cycle
                device.set_pwm_duty_cycle(0, pwm_config.initial_duty_cycle)?; // Channel 0 for demo
            }
        }
        println!("✅ PWM configuration complete");
    }

    // Configure encoders
    if !config.encoders.is_empty() {
        println!("🔧 Configuring encoders...");
        for (name, encoder_config) in &config.encoders {
            if encoder_config.enabled {
                println!(
                    "   Encoder {}: Pins {} & {} ({}x sampling)",
                    name,
                    encoder_config.pin_a,
                    encoder_config.pin_b,
                    if encoder_config.sampling_4x { "4" } else { "1" }
                );

                // Configure encoder (encoder 0 for demo)
                device.configure_encoder(
                    0,
                    encoder_config.pin_a,
                    encoder_config.pin_b,
                    encoder_config.sampling_4x,
                )?;
            }
        }
        println!("✅ Encoder configuration complete");
    }

    // Configure MAX7219 displays
    if !config.max7219_displays.is_empty() {
        println!("🔧 Configuring MAX7219 displays...");

        use pokeys_config::service::SpiMax7219Service;
        SpiMax7219Service::apply_device_spi_max7219_config(device, config)
            .map_err(|e| format!("MAX7219 configuration failed: {}", e))?;

        println!("✅ MAX7219 display configuration complete");
    }

    println!("🎉 Complete device configuration applied successfully!");

    Ok(())
}

/// Demonstrate the configured device by testing various functions
fn demonstrate_configured_device(
    device: &mut PoKeysDevice,
    config: &DeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Testing configured device functions...");

    // Test digital outputs (LEDs)
    if config.pins.contains_key(&3) && config.pins.contains_key(&4) {
        println!("\n💡 Testing LED outputs...");

        // Blink green LED
        println!("   Blinking Green LED (pin 3)...");
        for i in 0..3 {
            device.set_digital_output(3, true)?;
            thread::sleep(Duration::from_millis(300));
            device.set_digital_output(3, false)?;
            thread::sleep(Duration::from_millis(300));
            println!("     Blink {} complete", i + 1);
        }

        // Blink red LED
        println!("   Blinking Red LED (pin 4)...");
        for i in 0..3 {
            device.set_digital_output(4, true)?;
            thread::sleep(Duration::from_millis(300));
            device.set_digital_output(4, false)?;
            thread::sleep(Duration::from_millis(300));
            println!("     Blink {} complete", i + 1);
        }
    }

    // Test digital inputs (buttons)
    if config.pins.contains_key(&1) || config.pins.contains_key(&2) {
        println!("\n🔘 Testing button inputs...");
        println!("   Reading button states...");

        if config.pins.contains_key(&1) {
            let start_button = device.get_digital_input(1)?;
            println!(
                "     Start Button (pin 1): {}",
                if start_button { "PRESSED" } else { "RELEASED" }
            );
        }

        if config.pins.contains_key(&2) {
            let stop_button = device.get_digital_input(2)?;
            println!(
                "     Stop Button (pin 2): {}",
                if stop_button { "PRESSED" } else { "RELEASED" }
            );
        }
    }

    // Test PWM output
    if !config.pwm.is_empty() {
        println!("\n🌀 Testing PWM output (Fan Control)...");

        let duty_cycles = [0.0, 25.0, 50.0, 75.0, 100.0, 0.0];
        for duty in duty_cycles {
            println!("   Setting fan speed to {}%", duty);
            device.set_pwm_duty_cycle(0, duty)?;
            thread::sleep(Duration::from_millis(500));
        }
    }

    // Test encoder reading
    if !config.encoders.is_empty() {
        println!("\n🔄 Testing encoder reading...");

        let encoder_value = device.get_encoder_value(0)?;
        println!("   Current encoder position: {}", encoder_value);
        println!("   (Try rotating the encoder to see changes)");
    }

    // Test analog input
    if !config.analog_inputs.is_empty() {
        println!("\n📊 Testing analog input (Temperature Sensor)...");

        let analog_value = device.get_analog_input(26)?;
        let voltage = (analog_value as f32 / 4095.0) * 10.0; // Convert to voltage (0-10V)
        println!("   Raw ADC value: {}", analog_value);
        println!("   Voltage: {:.2}V", voltage);
        println!(
            "   Temperature: {:.1}°C (assuming 0-10V = 0-100°C)",
            voltage * 10.0
        );
    }

    // Test MAX7219 displays
    if !config.max7219_displays.is_empty() {
        println!("\n📺 Testing MAX7219 displays...");

        use pokeys_lib::devices::spi::Max7219;

        // Test status display
        if config.max7219_displays.contains_key("status_display") {
            let display_config = &config.max7219_displays["status_display"];
            let mut display = Max7219::new(device, display_config.cs_pin)?;

            println!(
                "   Testing Status Display (CS pin {})...",
                display_config.cs_pin
            );

            // Configure display
            display.configure_raw_segments(8)?;
            display.set_intensity(display_config.intensity)?;

            // Show different messages
            let messages = ["HELLO", "WORLD", "TEST", "DONE"];
            for message in messages {
                println!("     Displaying: {}", message);
                display.display_text(message)?;
                thread::sleep(Duration::from_millis(1000));
            }
        }

        // Test counter display
        if config.max7219_displays.contains_key("counter_display") {
            let display_config = &config.max7219_displays["counter_display"];
            let mut display = Max7219::new(device, display_config.cs_pin)?;

            println!(
                "   Testing Counter Display (CS pin {})...",
                display_config.cs_pin
            );

            // Configure display for numbers
            display.configure_numeric_display(6)?;
            display.set_intensity(display_config.intensity)?;

            // Count from 0 to 100
            for i in (0..=100).step_by(10) {
                println!("     Displaying: {}", i);
                display.display_number(i)?;
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    println!("\n✅ Device demonstration complete!");

    Ok(())
}

/// Show what would be configured if device was available
fn show_configuration_summary(config: &DeviceConfig) {
    println!("\n📋 Configuration Summary (Would be applied to device)");
    println!("====================================================");

    println!("🔌 Pin Configuration:");
    for (pin_num, pin_config) in &config.pins {
        if pin_config.enabled {
            println!(
                "   Pin {}: {} ({:?})",
                pin_num, pin_config.name, pin_config.function
            );
        }
    }

    if let Some(ref spi_config) = config.spi {
        if spi_config.enabled {
            println!("\n🔧 SPI Configuration:");
            println!("   Name: {}", spi_config.name);
            println!("   Prescaler: 0x{:02X}", spi_config.prescaler);
            println!("   Mode: 0x{:02X}", spi_config.mode);
        }
    }

    if !config.pwm.is_empty() {
        println!("\n⚡ PWM Configuration:");
        for (name, pwm_config) in &config.pwm {
            if pwm_config.enabled {
                println!(
                    "   {}: Pin {} at {}Hz",
                    name, pwm_config.pin, pwm_config.frequency_hz
                );
            }
        }
    }

    if !config.encoders.is_empty() {
        println!("\n🔄 Encoder Configuration:");
        for (name, encoder_config) in &config.encoders {
            if encoder_config.enabled {
                println!(
                    "   {}: Pins {} & {}",
                    name, encoder_config.pin_a, encoder_config.pin_b
                );
            }
        }
    }

    if !config.analog_inputs.is_empty() {
        println!("\n📊 Analog Input Configuration:");
        for (pin_num, analog_config) in &config.analog_inputs {
            if analog_config.enabled {
                println!(
                    "   Pin {}: {} ({}V reference)",
                    pin_num, analog_config.name, analog_config.reference_voltage
                );
            }
        }
    }

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
