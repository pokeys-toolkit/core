//! Configuration Loader Example
//!
//! This example shows basic device configuration and control
//! without external configuration dependencies.

use pokeys_lib::*;
use pokeys_lib::devices::spi::Max7219;
use std::thread;
use std::time::Duration;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📁 Configuration Loader Example");
    println!("===============================");

    // Enumerate devices
    let device_count = enumerate_usb_devices()?;
    if device_count == 0 {
        return Err("No PoKeys devices found! Please connect a device and try again.".into());
    }

    println!("✅ Found {} device(s)", device_count);

    // Connect to first device
    let mut device = connect_to_device(0)?;
    let _device_info = device.get_device_data()?;
    println!("📱 Connected to device");

    // Apply basic configuration
    apply_basic_configuration(&mut device)?;
    
    // Test the configured device
    test_device_functions(&mut device)?;
    
    // Cleanup
    cleanup_device(&mut device)?;

    println!("✅ Example completed successfully!");
    Ok(())
}

fn apply_basic_configuration(device: &mut PoKeysDevice) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Applying configuration...");
    
    // Configure digital I/O
    device.set_pin_function(1, PinFunction::DigitalInput)?;  // Button
    device.set_pin_function(3, PinFunction::DigitalOutput)?; // LED 1
    device.set_pin_function(4, PinFunction::DigitalOutput)?; // LED 2
    device.set_digital_output(3, false)?;
    device.set_digital_output(4, false)?;
    
    // Configure PWM
    device.configure_pwm_channel(0, 5, 0.0, true)?; // Channel 0, pin 5, 0% duty, enabled
    
    // Configure MAX7219 display if possible
    let mut display = Max7219::new(device, 24)?;
    display.configure_numeric(8)?;
    display.set_intensity(8)?;
    display.display_text("READY")?;
    
    println!("✅ Configuration applied");
    Ok(())
}

fn test_device_functions(device: &mut PoKeysDevice) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🎮 Testing device functions...");
    
    for i in 0..10 {
        // Read button
        let button = device.get_digital_input(1)?;
        
        // Control LEDs based on button and counter
        device.set_digital_output(3, button || i % 2 == 0)?;
        device.set_digital_output(4, !button && i % 3 == 0)?;
        
        // Vary PWM
        let duty = (i * 10) % 100;
        device.set_pwm_duty_cycle(0, duty as u32)?;
        
        if button {
            println!("   Button pressed! LEDs responding...");
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    Ok(())
}

fn cleanup_device(device: &mut PoKeysDevice) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🧹 Cleaning up...");
    
    // Turn off all outputs
    device.set_digital_output(3, false)?;
    device.set_digital_output(4, false)?;
    device.set_pwm_duty_cycle(0, 0)?;
    
    // Clear display if configured
    let mut display = Max7219::new(device, 24)?;
    display.clear()?;
    
    println!("✅ Cleanup complete");
    Ok(())
}
