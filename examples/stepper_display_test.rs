use pokeys_lib::*;
use std::{thread::current, time::Duration};

fn main() -> Result<()> {
    println!("=== Stepper Motor Display Test ===");

    // Connect to network device with serial 32223
    let network_devices = enumerate_network_devices(5000)?; // 5 second timeout
    let target_device = network_devices
        .iter()
        .find(|d| d.serial_number == 32223)
        .ok_or_else(|| PoKeysError::Parameter("Device with serial 32223 not found".to_string()))?;

    let mut device = connect_to_network_device(target_device)?;
    println!("✓ Connected to network device (serial: 32223)");

    // Configure pulse engine with 3 axes
    device.enable_pulse_engine(true)?;

    let config = PulseEngineConfig::three_channel_internal(3, false)
        .power_states(0x06) // Only enable power for STOP_LIMIT and STOP_EMERGENCY, NOT STOPPED
        .build();

    let axis_enabled_mask = 0x04;
    device.setup_pulse_engine_with_axes(&config, axis_enabled_mask)?;

    // Set pulse engine state
    device.set_pulse_engine_state(0x02, 0x00, axis_enabled_mask)?;

    println!("✓ Pulse engine configured");

    // Configure axis 1 (index 0) for stepper motor
    device
        .configure_axis(2) // Axis 1 (index 0)
        .max_speed(25000.0)
        .max_acceleration(10.0)
        .max_deceleration(10.0)
        .soft_limit_min(-100000)
        .soft_limit_max(100000)
        .step_angle(1.8)
        .step_resolution(pulse_engine::StepResolution::SixteenthStep)
        .build(&mut device)?;

    device.set_motor_drivers_configuration()?;
    device.set_axis_configuration(2)?;
    println!("✓ Axis 1 configured for stepper motor");

    // Initialize I2C for uSPIBridge communication
    device.i2c_init()?;
    println!("✓ I2C initialized for uSPIBridge");

    // Reset position to 0
    device.set_axis_position(2, 0)?;
    device.enable_pulse_engine(true)?;

    std::thread::sleep(Duration::from_millis(100));
    println!("✓ Position reset to 0");

    // Test sequence: 0 → -5000 → 5000 → 0
    let positions = [-1000, 1000, -1000, 1000, 1, 2, 3, 30, 300, 3000];

    for &target_pos in &positions {
        println!("\n--- Setting position to {} ---", target_pos);

        // Set axis position directly
        match device.move_axis_to_position(2, target_pos, 100.0) {
            Ok(_) => println!("✓ Position set successfully"),
            Err(e) => println!("✗ Set position failed: {:?}", e),
        }

        loop {
            // Get pulse engine status to update position data
            device.get_pulse_engine_status()?;
            let current_pos = device.pulse_engine_v2.current_position[2];

            let _ = update_display(&mut device, current_pos, target_pos);

            if current_pos == target_pos {
                break;
            }

            std::thread::sleep(Duration::from_millis(50));
        }
    }

    // Clean up
    println!("\n✓ Stepper motor test completed");
    Ok(())
}

fn update_display(device: &mut PoKeysDevice, current_pos: i32, target_pos: i32) -> Result<()> {
    // Format position to 6 digits with leading spaces
    let display_text = format!("{current_pos}");
    println!("---> {display_text}");

    // Send to standard uSPIBridge display (device 0)
    match device.uspibridge_display_text(0x42, 0, &display_text) {
        Ok(_) => {}
        Err(e) => println!("Display update failed: {:?}", e),
    }

    let target = format!("{target_pos}");

    match device.uspibridge_display_text(0x42, 1, &target) {
        Ok(_) => {}
        Err(e) => println!("Display update failed: {:?}", e),
    }

    Ok(())
}
