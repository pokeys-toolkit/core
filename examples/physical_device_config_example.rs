//! Physical Device Configuration Example
//!
//! This example demonstrates basic device configuration and control
//! without external dependencies.

use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Physical Device Configuration Example");
    println!("=======================================");

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

    // Basic pin configuration
    println!("\n🔧 Configuring pins...");

    // Configure pin 1 as digital output (LED)
    device.set_pin_function(1, PinFunction::DigitalOutput)?;
    device.set_digital_output(1, false)?;
    println!("   Pin 1: Digital Output (LED)");

    // Configure pin 2 as digital input (button)
    device.set_pin_function(2, PinFunction::DigitalInput)?;
    println!("   Pin 2: Digital Input (Button)");

    // Configure PWM on pin 5
    device.configure_pwm_channel(0, 5, 0.0, true)?; // Channel 0, pin 5, 0% duty, enabled
    println!("   Pin 5: PWM Output (Fan Control)");

    // Additional configuration can be added here as needed

    println!("\n🎮 Running demonstration...");

    // Run demonstration for 10 seconds
    for i in 0..20 {
        // Blink LED
        device.set_digital_output(1, i % 2 == 0)?;

        // Read button
        let button_state = device.get_digital_input(2)?;
        if button_state {
            println!("   Button pressed!");
        }

        // Vary PWM duty cycle
        let duty = (i * 5) % 100;
        device.set_pwm_duty_cycle(0, duty as u32)?;

        thread::sleep(Duration::from_millis(500));
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    device.set_digital_output(1, false)?;
    device.set_pwm_duty_cycle(0, 0)?;

    println!("✅ Example completed successfully!");
    Ok(())
}
