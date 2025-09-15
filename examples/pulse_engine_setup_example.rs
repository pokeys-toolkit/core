use pokeys_lib::pulse_engine::step_setting;
use pokeys_lib::*;
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("PoKeys Pulse Engine Setup Example");
    println!("=================================");

    // Connect to device 32223
    let network_devices = enumerate_network_devices(2000)?;
    let target_device = network_devices
        .iter()
        .find(|device| device.serial_number == 32223)
        .ok_or_else(|| PoKeysError::Parameter("Device 32223 not found".to_string()))?;

    let mut device = connect_to_network_device(target_device)?;

    // Configure for 3-axis internal pulse generator
    println!("Configuring 3-axis internal pulse generator...");

    device.enable_pulse_engine(false)?;

    let config = PulseEngineConfig::three_channel_internal(3, false).build();
    device.setup_pulse_engine(&config)?;

    // Enable axis 2 specifically (bit 1 for axis 2, 0-indexed)
    let axis_enabled_mask = 1 << 1; // Enable axis 2 (0-indexed)

    // Send the pulse engine state with axis 2 enabled
    device.set_pulse_engine_state(0x01, 0x00, axis_enabled_mask)?;

    // Send axis configuration after pulse engine state
    device.set_axis_configuration(2)?;

    // Read back the state to update local values
    device.get_pulse_engine_state()?;

    device.enable_pulse_engine(true)?;

    // Verify configuration
    device.get_pulse_engine_status()?;
    println!(
        "✓ Pulse engine configured: {} axes, generator type 0x{:02X}",
        device.pulse_engine_v2.info.nr_of_axes, device.pulse_engine_v2.pulse_generator_type
    );

    // Configure axis 2
    println!("Configuring axis 2...");
    device
        .configure_axis(2)
        .max_speed(10000)
        .max_acceleration(5000)
        .max_deceleration(5000)
        .soft_limit_min(-1800)
        .soft_limit_max(18000)
        .build(&mut device)?;

    // Read back configuration to verify
    device.get_axis_configuration(2)?;
    println!(
        "✓ Axis 2 configured: speed={}, accel={}, decel={}, limits=[{}, {}]",
        device.pulse_engine_v2.max_speed[2] as u32,
        device.pulse_engine_v2.max_acceleration[2] as u32,
        device.pulse_engine_v2.max_deceleration[2] as u32,
        device.pulse_engine_v2.soft_limit_minimum[2],
        device.pulse_engine_v2.soft_limit_maximum[2]
    );

    // Get motor driver configuration
    println!("Reading motor driver configuration...");
    device.get_motor_drivers_configuration()?;
    let step_names = [
        "1/1",
        "1/2 non-circular",
        "1/2",
        "1/4",
        "1/8",
        "1/16",
        "1/32",
        "1/128",
        "1/256",
    ];
    let step_setting = device.pulse_engine_v2.motor_step_setting[1]; // Axis 2 (0-indexed)
    let current_setting = device.pulse_engine_v2.motor_current_setting[1];
    let current_amps = (current_setting as f32) * 2.5 / 255.0;

    println!(
        "✓ Axis 2 motor driver: step={} ({}), current={:.2}A",
        step_setting,
        step_names.get(step_setting as usize).unwrap_or(&"Unknown"),
        current_amps
    );

    // Set axis 2 to 1/16 step setting
    println!("Setting axis 2 to 1/16 step setting...");
    device
        .configure_motor_drivers()
        .axis_step_setting(1, step_setting::SIXTEENTH_STEP) // Axis 2 (0-indexed), 1/16
        .build(&mut device)?;

    // Read back to verify
    let new_step_setting = device.pulse_engine_v2.motor_step_setting[1];
    println!(
        "✓ Axis 2 motor driver updated: step={} ({})",
        new_step_setting,
        step_names
            .get(new_step_setting as usize)
            .unwrap_or(&"Unknown")
    );

    // Interactive move command
    println!("\n--- Interactive Move Command ---");
    print!("Enter position for axis 2 (-180 to 180): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let position: i32 = input.trim().parse().unwrap_or(0);

    println!("Setting axis 2 to position {}...", position);

    // Enable pulse engine before moving
    device.enable_pulse_engine(true)?;

    // Check axis state before moving
    let axis_state = device.get_axis_state(2)?;
    println!("Axis 2 state before move: {:?}", axis_state);
    println!(
        "Axis 2 enabled: {}",
        device.pulse_engine_v2.is_axis_enabled(2)
    );

    // Get current position
    let current_pos = device.get_axis_position(2)?;
    println!("Current position: {}", current_pos);

    // Use the existing move_axis_to_position method
    device.move_axis_to_position(2, position, 50.0)?; // 50% speed
    println!("✓ Move command sent");

    // Check state after move command
    let axis_state_after = device.get_axis_state(2)?;
    println!("Axis 2 state after move: {:?}", axis_state_after);

    Ok(())
}
