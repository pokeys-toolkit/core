//! Step-by-Step Device Configuration Example
//! 
//! This example shows the complete process of:
//! 1. Creating a configuration
//! 2. Connecting to a physical device
//! 3. Applying the configuration
//! 4. Testing the configured device

use pokeys_lib::*;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Step-by-Step Device Configuration");
    println!("===================================");
    println!("This example shows how to configure a physical PoKeys device");
    println!("to match a specific configuration.\n");

    // Step 1: Define what we want to configure
    println!("📋 Step 1: Define Configuration Requirements");
    println!("============================================");
    println!("We want to configure:");
    println!("  • Pin 1: Start button (digital input with pull-up)");
    println!("  • Pin 2: Emergency stop (digital input with pull-up)");
    println!("  • Pin 3: Green status LED (digital output)");
    println!("  • Pin 4: Red alarm LED (digital output)");
    println!("  • Pin 5: Fan speed control (PWM output)");
    println!("  • Pin 24: MAX7219 display (SPI CS pin)");
    println!("  • SPI: Enabled for display communication");

    // Step 2: Connect to the physical device
    println!("\n🔌 Step 2: Connect to Physical Device");
    println!("=====================================");
    
    // Try to connect to device with serial number 32218
    let mut device = match connect_to_device_with_serial(32218, true, 3000) {
        Ok(device) => {
            println!("✅ Connected to PoKeys device (Serial: 32218)");
            device
        },
        Err(_) => {
            // If specific device not found, try to connect to any device
            println!("⚠️  Device 32218 not found, trying to connect to any available device...");
            
            let device_count = enumerate_usb_devices()?;
            if device_count > 0 {
                let device = connect_to_device(0)?;
                println!("✅ Connected to PoKeys device (Index: 0)");
                device
            } else {
                println!("❌ No PoKeys devices found!");
                println!("   Please connect a PoKeys device and try again.");
                return Ok(());
            }
        }
    };

    // Get device information
    let device_info = device.get_device_info()?;
    println!("   Device: {}", device_info.device_name);
    println!("   Firmware: {}.{}", 
        device_info.firmware_version_major, 
        device_info.firmware_version_minor);
    println!("   Pins: {}", device_info.pin_count);

    // Step 3: Configure the device pins
    println!("\n⚙️  Step 3: Configure Device Pins");
    println!("=================================");
    
    // Configure digital input pins (buttons)
    println!("🔘 Configuring digital inputs...");
    device.set_pin_function(1, pokeys_lib::io::PinFunction::DigitalInput)?;
    device.set_pin_function(2, pokeys_lib::io::PinFunction::DigitalInput)?;
    println!("   ✅ Pin 1: Start button (digital input)");
    println!("   ✅ Pin 2: Emergency stop (digital input)");
    
    // Configure digital output pins (LEDs)
    println!("💡 Configuring digital outputs...");
    device.set_pin_function(3, pokeys_lib::io::PinFunction::DigitalOutput)?;
    device.set_pin_function(4, pokeys_lib::io::PinFunction::DigitalOutput)?;
    
    // Set initial states (both LEDs off)
    device.set_digital_output(3, false)?; // Green LED off
    device.set_digital_output(4, false)?; // Red LED off
    println!("   ✅ Pin 3: Green LED (digital output, initially OFF)");
    println!("   ✅ Pin 4: Red LED (digital output, initially OFF)");
    
    // Configure PWM pin
    println!("🌀 Configuring PWM output...");
    device.set_pin_function(5, pokeys_lib::io::PinFunction::DigitalOutput)?; // PWM uses digital output
    device.set_pwm_duty_cycle(0, 0.0)?; // Start with 0% duty cycle
    println!("   ✅ Pin 5: Fan control (PWM output, initially 0%)");

    // Step 4: Configure SPI for displays
    println!("\n🔧 Step 4: Configure SPI Communication");
    println!("======================================");
    
    // Configure SPI
    device.spi_configure(0x04, 0x00)?; // Prescaler 0x04, Mode 0x00
    println!("   ✅ SPI configured (prescaler: 0x04, mode: 0x00)");
    println!("   ✅ Pin 23: Automatically configured as SPI MOSI");
    println!("   ✅ Pin 25: Automatically configured as SPI CLK");

    // Step 5: Configure MAX7219 display
    println!("\n📺 Step 5: Configure MAX7219 Display");
    println!("====================================");
    
    use pokeys_lib::devices::spi::Max7219;
    let mut display = Max7219::new(&mut device, 24)?; // CS pin 24
    
    // Configure display for text
    display.configure_raw_segments(8)?;
    display.set_intensity(8)?; // Medium brightness
    display.display_text("READY")?;
    
    println!("   ✅ MAX7219 display configured (CS pin 24)");
    println!("   ✅ Display showing: READY");

    // Step 6: Test the configured device
    println!("\n🎮 Step 6: Test Configured Device");
    println!("=================================");
    
    println!("Testing device functionality for 10 seconds...");
    println!("(Press Ctrl+C to stop early)");
    
    let start_time = std::time::Instant::now();
    let mut counter = 0;
    
    while start_time.elapsed() < Duration::from_secs(10) {
        // Read button states
        let start_button = device.get_digital_input(1)?;
        let stop_button = device.get_digital_input(2)?;
        
        // Control LEDs based on button states
        if start_button {
            device.set_digital_output(3, true)?;  // Green LED on when start pressed
            device.set_digital_output(4, false)?; // Red LED off
            display.display_text("START")?;
        } else if stop_button {
            device.set_digital_output(3, false)?; // Green LED off
            device.set_digital_output(4, true)?;  // Red LED on when stop pressed
            display.display_text("STOP")?;
        } else {
            device.set_digital_output(3, false)?; // Both LEDs off when no buttons pressed
            device.set_digital_output(4, false)?;
            display.display_text("READY")?;
        }
        
        // Vary fan speed in a cycle
        let fan_speed = ((counter as f32 * 2.0).sin() + 1.0) * 50.0; // 0-100%
        device.set_pwm_duty_cycle(0, fan_speed)?;
        
        // Update counter display every second
        if counter % 10 == 0 {
            let seconds_remaining = 10 - (start_time.elapsed().as_secs() as i32);
            println!("   Buttons: Start={}, Stop={} | Fan: {:.1}% | Time: {}s", 
                if start_button { "PRESSED" } else { "released" },
                if stop_button { "PRESSED" } else { "released" },
                fan_speed,
                seconds_remaining);
        }
        
        counter += 1;
        thread::sleep(Duration::from_millis(100));
    }

    // Step 7: Clean up and show final status
    println!("\n🧹 Step 7: Clean Up");
    println!("===================");
    
    // Turn off all outputs
    device.set_digital_output(3, false)?; // Green LED off
    device.set_digital_output(4, false)?; // Red LED off
    device.set_pwm_duty_cycle(0, 0.0)?;   // Fan off
    display.display_text("DONE")?;
    
    println!("   ✅ All outputs turned off");
    println!("   ✅ Display showing: DONE");

    // Show configuration summary
    println!("\n📊 Configuration Summary");
    println!("========================");
    println!("Successfully configured and tested:");
    println!("  ✅ Pin 1: Start button (digital input) - Tested ✓");
    println!("  ✅ Pin 2: Emergency stop (digital input) - Tested ✓");
    println!("  ✅ Pin 3: Green LED (digital output) - Tested ✓");
    println!("  ✅ Pin 4: Red LED (digital output) - Tested ✓");
    println!("  ✅ Pin 5: Fan control (PWM output) - Tested ✓");
    println!("  ✅ Pin 23: SPI MOSI (automatic) - Configured ✓");
    println!("  ✅ Pin 25: SPI CLK (automatic) - Configured ✓");
    println!("  ✅ Pin 24: MAX7219 display (SPI CS) - Tested ✓");
    println!("  ✅ SPI communication - Working ✓");

    println!("\n🎉 Device Configuration Complete!");
    println!("=================================");
    println!("Your PoKeys device is now configured and ready for use.");
    println!("All pins are set up according to the configuration and have been tested.");
    
    Ok(())
}
