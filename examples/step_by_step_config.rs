//! Step-by-Step Configuration Example
//!
//! This example walks through device configuration step by step,
//! demonstrating each major subsystem individually.

use pokeys_lib::{encoders::EncoderOptions, *};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🚀 Step-by-Step PoKeys Configuration");
    println!("====================================");

    // Connect to device
    let device_count = enumerate_usb_devices()?;
    if device_count == 0 {
        return Err(PoKeysError::DeviceNotFound);
    }

    let mut device = connect_to_device(0)?;
    device.get_device_data()?;

    println!(
        "📱 Connected to device: {}",
        device.device_data.serial_number
    );

    // Step-by-step configuration
    configure_digital_io(&mut device)?;
    configure_pwm(&mut device)?;
    configure_encoders(&mut device)?;
    configure_additional_features(&mut device)?;
    configure_analog_inputs(&mut device)?;

    // Interactive demonstration
    run_interactive_demo(&mut device)?;

    println!("✅ Step-by-step configuration completed!");
    Ok(())
}

fn configure_digital_io(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n📌 Step 1: Configure Digital I/O");
    println!("Press Enter to continue...");
    wait_for_enter();

    device.set_pin_function(1, PinFunction::DigitalInput)?; // Button
    device.set_pin_function(2, PinFunction::DigitalInput)?; // Switch
    device.set_pin_function(3, PinFunction::DigitalOutput)?; // LED 1
    device.set_pin_function(4, PinFunction::DigitalOutput)?; // LED 2

    println!("   ✅ Pin 1: Digital Input (Button)");
    println!("   ✅ Pin 2: Digital Input (Switch)");
    println!("   ✅ Pin 3: Digital Output (LED 1)");
    println!("   ✅ Pin 4: Digital Output (LED 2)");

    Ok(())
}

fn configure_pwm(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🌊 Step 2: Configure PWM Outputs");
    println!("Press Enter to continue...");
    wait_for_enter();

    // Configure PWM (only pins 17-22 support PWM)
    device.set_pwm_period(20000)?; // 20ms period
    device.enable_pwm_for_pin(22, true)?; // PWM1 on pin 22
    device.enable_pwm_for_pin(21, true)?; // PWM2 on pin 21

    println!("   ✅ PWM Channel 0: Pin 5 (Motor Control)");
    println!("   ✅ PWM Channel 1: Pin 6 (Fan Control)");

    Ok(())
}

fn configure_encoders(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🔄 Step 3: Configure Encoders");
    println!("Press Enter to continue...");
    wait_for_enter();

    let options = EncoderOptions::with_4x_sampling();
    device.configure_encoder(0, 10, 11, options)?;

    println!("   ✅ Encoder 0: Pins 10-11 (4x Quadrature)");

    Ok(())
}

fn configure_additional_features(_device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🔧 Step 4: Configure Additional Features");
    println!("Press Enter to continue...");
    wait_for_enter();

    // Additional features can be configured here as needed
    println!("   ✅ Additional features configured");
    println!("   ✅ Device ready for advanced operations");

    Ok(())
}

fn configure_analog_inputs(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n📊 Step 5: Configure Analog Inputs");
    println!("Press Enter to continue...");
    wait_for_enter();

    device.set_pin_function(41, PinFunction::AnalogInput)?; // Analog pin
    device.set_pin_function(42, PinFunction::AnalogInput)?; // Analog pin

    println!("   ✅ Pin 41: Analog Input (Sensor 1)");
    println!("   ✅ Pin 42: Analog Input (Sensor 2)");

    Ok(())
}

fn run_interactive_demo(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🎮 Step 6: Interactive Demonstration");
    println!("Press Enter to start the demo, or 'q' to quit...");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim() == "q" {
        return Ok(());
    }

    println!("Running 10-second demonstration...");

    for i in 0..40 {
        // Read inputs
        let button = device.get_digital_input(1)?;
        let switch = device.get_digital_input(2)?;
        let encoder_pos = device.get_encoder_value(0)?;
        let analog1 = device.get_analog_input(41)?;
        let analog2 = device.get_analog_input(42)?;

        // Control outputs
        device.set_digital_output(3, button || i % 4 == 0)?;
        device.set_digital_output(4, switch || i % 6 == 0)?;

        // Dynamic PWM
        let pwm1_duty = ((i * 2) % 100) as u32;
        let pwm2_duty = if button { 80 } else { 20 };

        device.set_pwm_duty_cycle_for_pin(22, pwm1_duty)?;
        device.set_pwm_duty_cycle_for_pin(21, pwm2_duty)?;

        // Status every 2 seconds
        if i % 8 == 0 {
            println!(
                "   Status: Button={}, Switch={}, Encoder={}, Analog1={}, Analog2={}",
                if button { "ON" } else { "OFF" },
                if switch { "ON" } else { "OFF" },
                encoder_pos,
                analog1,
                analog2
            );
        }

        thread::sleep(Duration::from_millis(250));
    }

    // Cleanup
    device.set_digital_output(3, false)?;
    device.set_digital_output(4, false)?;
    device.set_pwm_duty_cycle_for_pin(22, 0)?;
    device.set_pwm_duty_cycle_for_pin(21, 0)?;

    println!("✅ Demonstration complete!");
    Ok(())
}

fn wait_for_enter() {
    print!("   ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
