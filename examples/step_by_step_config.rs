//! Step-by-Step Configuration Example
//!
//! This example walks through configuring a PoKeys device step by step,
//! demonstrating each major feature with clear explanations.

use pokeys_lib::*;
use pokeys_lib::devices::spi::Max7219;
use std::thread;
use std::time::Duration;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Step-by-Step PoKeys Configuration");
    println!("====================================");
    println!("This example will guide you through configuring a PoKeys device step by step.\n");

    // Step 1: Device Discovery
    println!("⚡ Step 1: Device Discovery");
    println!("==========================");
    let device_count = enumerate_usb_devices()?;
    println!("Found {} PoKeys device(s)", device_count);

    if device_count == 0 {
        return Err("No PoKeys devices found! Please connect a device and try again.".into());
    }

    // Step 2: Device Connection
    println!("\n🔌 Step 2: Device Connection");
    println!("============================");
    let mut device = connect_to_device(0)?;
    let _device_info = device.get_device_data()?;
    println!("   Device: Connected");
    println!("   Firmware: Available");
    println!("   Pins: Available");

    // Step 3: Configure the device pins
    println!("\n⚙️  Step 3: Configure Device Pins");
    println!("=================================");
    
    // Configure digital inputs (buttons)
    device.set_pin_function(1, PinFunction::DigitalInput)?;
    device.set_pin_function(2, PinFunction::DigitalInput)?;
    println!("   Pin 1: Digital Input (Start Button)");
    println!("   Pin 2: Digital Input (Stop Button)");

    // Configure digital outputs (LEDs)
    device.set_pin_function(3, PinFunction::DigitalOutput)?;
    device.set_pin_function(4, PinFunction::DigitalOutput)?;
    device.set_digital_output(3, false)?;
    device.set_digital_output(4, false)?;
    println!("   Pin 3: Digital Output (Green LED)");
    println!("   Pin 4: Digital Output (Red LED)");

    // Step 4: Configure PWM
    println!("\n🌊 Step 4: Configure PWM");
    println!("========================");
    device.configure_pwm_channel(0, 5, 0.0, true)?;
    println!("   PWM Channel 0: Pin 5 (Fan Control)");

    // Step 5: Configure MAX7219 Display
    println!("\n📺 Step 5: Configure MAX7219 Display");
    println!("====================================");
    {
        let mut display = Max7219::new(&mut device, 24)?;
        display.configure_numeric(8)?;
        display.set_intensity(8)?;
        display.display_text("READY")?;
        println!("   Display: 8-digit numeric on CS pin 24");
    }

    // Step 6: Interactive demonstration
    println!("\n🎮 Step 6: Interactive Demonstration");
    println!("====================================");
    println!("Testing device functionality for 10 seconds...");

    for i in 0..20 {
        // Read button states
        let start_button = device.get_digital_input(1)?;
        let stop_button = device.get_digital_input(2)?;

        // Control LEDs and display based on button states
        if start_button {
            device.set_digital_output(3, true)?;
            device.set_digital_output(4, false)?;
            {
                let mut display = Max7219::new(&mut device, 24)?;
                display.display_text("START")?;
            }
        } else if stop_button {
            device.set_digital_output(3, false)?;
            device.set_digital_output(4, true)?;
            {
                let mut display = Max7219::new(&mut device, 24)?;
                display.display_text("STOP")?;
            }
        } else {
            device.set_digital_output(3, false)?;
            device.set_digital_output(4, false)?;
            {
                let mut display = Max7219::new(&mut device, 24)?;
                display.display_text("READY")?;
            }
        }

        // Vary fan speed
        let fan_speed = (i * 5) % 100;
        device.set_pwm_duty_cycle_percent(0, fan_speed as f32)?;

        thread::sleep(Duration::from_millis(500));
    }

    // Step 7: Cleanup
    println!("\n🧹 Step 7: Cleanup");
    println!("==================");
    device.set_digital_output(3, false)?;
    device.set_digital_output(4, false)?;
    device.set_pwm_duty_cycle_percent(0, 0.0)?;
    {
        let mut display = Max7219::new(&mut device, 24)?;
        display.display_text("DONE")?;
    }

    println!("\n✅ Step-by-Step Configuration Complete!");
    println!("You have successfully configured and tested your PoKeys device.");
    
    Ok(())
}
