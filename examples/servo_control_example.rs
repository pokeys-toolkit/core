use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("PoKeys Servo Control Example");
    println!("===========================");

    match connect_to_first_available_device() {
        Ok(mut device) => run_servo_examples(&mut device),
        Err(_) => {
            println!("No PoKeys device found. Connect hardware and try again.");
            Ok(())
        }
    }
}

fn run_servo_examples(device: &mut PoKeysDevice) -> Result<()> {
    println!("✓ Connected to PoKeys device");

    // Example 1: 180-degree servo (standard servo)
    println!("\n--- 180-Degree Servo Example ---");
    let servo_180 = ServoConfig::one_eighty(22, 60000, 12000); // Pin 22, calibrated positions

    device.configure_servo(servo_180.clone())?;

    let angles = [0.0, 45.0, 90.0, 135.0, 180.0, 90.0];
    for angle in angles.iter() {
        println!("Moving to {}°", angle);
        device.set_servo_angle(&servo_180, *angle)?;
        thread::sleep(Duration::from_millis(800));
    }

    // Example 2: 360-degree speed servo (continuous rotation) on pin 21
    println!("\n--- 360-Degree Speed Servo Example (Pin 21) ---");
    // Datasheet: Counterclockwise 1-1.5ms, Stop 1.5ms, Clockwise 1.5-2ms
    // 1ms = 25,000 cycles, 1.5ms = 37,500 cycles, 2ms = 50,000 cycles
    let servo_360_speed = ServoConfig::three_sixty_speed(21, 37500, 50000, 25000); // stop, clockwise, counterclockwise

    device.configure_servo(servo_360_speed.clone())?;

    // Test different speeds
    let speeds = [
        0.0, 25.0, 50.0, 75.0, 100.0, 0.0, -25.0, -50.0, -75.0, -100.0, 0.0,
    ];
    for speed in speeds.iter() {
        if *speed == 0.0 {
            println!("Stopping servo");
            device.stop_servo(&servo_360_speed)?;
        } else {
            println!(
                "Setting speed to {}% ({})",
                speed,
                if *speed > 0.0 {
                    "clockwise"
                } else {
                    "anti-clockwise"
                }
            );
            device.set_servo_speed(&servo_360_speed, *speed)?;
        }
        thread::sleep(Duration::from_millis(1000));
    }

    // Disable all PWM channels
    for pin in [22, 21] {
        let _ = device.enable_pwm_for_pin(pin, false);
    }

    println!("\n✓ Servo examples complete");
    Ok(())
}

fn connect_to_first_available_device() -> Result<PoKeysDevice> {
    if enumerate_usb_devices()? > 0 {
        return connect_to_device(0);
    }

    let network_devices = enumerate_network_devices(1000)?;
    if !network_devices.is_empty() {
        return connect_to_network_device(&network_devices[0]);
    }

    Err(PoKeysError::DeviceNotFound)
}
