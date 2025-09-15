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

    let config = PulseEngineConfig::three_channel_internal(3, false)
        .power_states(0x06) // Only enable power for STOP_LIMIT and STOP_EMERGENCY, NOT STOPPED
        .build();

    // Enable axis 3 (bit 2) for testing
    let axis_enabled_mask = 0x04;

    // Debug: Show what we're sending to setup_pulse_engine
    println!("Setup pulse engine config:");
    println!("  enabled_axes: {}", config.enabled_axes);
    println!("  charge_pump_enabled: {}", config.charge_pump_enabled);
    println!("  generator_type: 0x{:02X}", config.generator_type);
    println!("  buffer_size: {}", config.buffer_size);
    println!(
        "  emergency_switch_polarity: {}",
        config.emergency_switch_polarity
    );
    println!("  power_states: 0x{:02X}", config.power_states);
    println!("  axis_enabled_mask: 0x{:02X}", axis_enabled_mask);

    device.setup_pulse_engine_with_axes(&config, axis_enabled_mask)?;

    // Reboot pulse engine to reset state
    device.reboot_pulse_engine()?;

    // Simple test - configure and enable axis 3 (index 2, bit 2)
    println!("Simple test - enabling axis 3 (bit 2)...");

    // Configure axis 3 (index 2) - the one we're actually enabling
    device
        .configure_axis(2)
        .max_speed(100)
        .max_acceleration(50)
        .max_deceleration(50)
        .soft_limit_min(-1800)
        .soft_limit_max(1800)
        .build(&mut device)?;
    device.set_axis_configuration(2)?;

    // Try to enable axis 3 (bit 2) - match what we set in setup
    device.set_pulse_engine_state(0x02, 0x00, axis_enabled_mask)?;
    device.enable_pulse_engine(true)?;
    device.get_pulse_engine_status()?;

    // Get detailed status using 0x85/0x00
    println!("Getting pulse engine status (0x85/0x00)...");
    device.get_pulse_engine_status()?;

    println!("Pulse engine status:");
    println!(
        "  pulse_engine_activated: {}",
        device.pulse_engine_v2.pulse_engine_activated
    );
    println!(
        "  pulse_engine_state: {}",
        device.pulse_engine_v2.pulse_engine_state
    );
    println!(
        "  axis_enabled_states_mask: 0x{:02X}",
        device.pulse_engine_v2.axis_enabled_states_mask
    );
    println!(
        "  axis_enabled_mask: 0x{:02X}",
        device.pulse_engine_v2.axis_enabled_mask
    );
    println!("  nr_of_axes: {}", device.pulse_engine_v2.info.nr_of_axes);

    // Show axis positions from 0x85/0x00
    println!("Axis positions from status:");
    for i in 0..3 {
        println!(
            "  Axis {} position: {}",
            i + 1,
            device.pulse_engine_v2.current_position[i]
        );
    }

    // Use bit 0 for axis 1
    let axis_enabled_mask = 0x01;

    // Enable pulse engine FIRST
    device.enable_pulse_engine(true)?;

    // Send the pulse engine state with axis 3 enabled - try state 0x02 (running) instead of 0x01
    device.set_pulse_engine_state(0x02, 0x00, axis_enabled_mask)?;

    // Send axis configuration after pulse engine state
    device.set_axis_configuration(2)?;

    // Read back the status to update local values
    device.get_pulse_engine_status()?;

    // Debug: Check what the device reports as enabled
    println!(
        "Device axis_enabled_mask: 0b{:08b} (0x{:02X})",
        device.pulse_engine_v2.axis_enabled_mask, device.pulse_engine_v2.axis_enabled_mask
    );
    println!(
        "Expected mask for axis 3: 0b{:08b} (0x{:02X})",
        axis_enabled_mask, axis_enabled_mask
    );
    println!(
        "Bit 0 (axis 1): {}",
        (device.pulse_engine_v2.axis_enabled_mask & 1) != 0
    );
    println!(
        "Bit 1 (axis 2): {}",
        (device.pulse_engine_v2.axis_enabled_mask & 2) != 0
    );
    println!(
        "Bit 2 (axis 3): {}",
        (device.pulse_engine_v2.axis_enabled_mask & 4) != 0
    );

    // Check if axis 3 (index 2) is now enabled
    println!(
        "Axis 3 enabled after config: {}",
        device.pulse_engine_v2.is_axis_enabled(2)
    );

    device.enable_pulse_engine(true)?;

    // Verify configuration
    device.get_pulse_engine_status()?;
    println!(
        "✓ Pulse engine configured: {} axes, generator type 0x{:02X}",
        device.pulse_engine_v2.info.nr_of_axes, device.pulse_engine_v2.pulse_generator_type
    );

    // Configure axis 3 (index 2)
    println!("Configuring axis 3...");
    device
        .configure_axis(2)
        .max_speed(10000)
        .max_acceleration(5000)
        .max_deceleration(5000)
        .soft_limit_min(-1800)
        .soft_limit_max(1800)
        .build(&mut device)?;

    // Read back configuration to verify
    device.get_axis_configuration(2)?;
    println!(
        "✓ Axis 3 configured: speed={}, accel={}, decel={}, limits=[{}, {}]",
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
    let step_setting = device.pulse_engine_v2.motor_step_setting[2]; // Axis 3 (index 2)
    let current_setting = device.pulse_engine_v2.motor_current_setting[2];
    let current_amps = (current_setting as f32) * 2.5 / 255.0;

    println!(
        "✓ Axis 3 motor driver: step={} ({}), current={:.2}A",
        step_setting,
        step_names.get(step_setting as usize).unwrap_or(&"Unknown"),
        current_amps
    );

    // Set axis 2 to 1/16 step setting
    println!("Setting axis 3 to 1/16 step setting...");
    device
        .configure_motor_drivers()
        .axis_step_setting(2, step_setting::SIXTEENTH_STEP) // Axis 3 (index 2), 1/16
        .build(&mut device)?;

    // Read back to verify
    let new_step_setting = device.pulse_engine_v2.motor_step_setting[2];
    println!(
        "✓ Axis 3 motor driver updated: step={} ({})",
        new_step_setting,
        step_names
            .get(new_step_setting as usize)
            .unwrap_or(&"Unknown")
    );

    // Interactive move command
    println!("\n--- Interactive Move Command ---");
    print!("Enter position for axis 3 (-180 to 180): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let position: i32 = input.trim().parse().unwrap_or(0);

    println!("Setting axis 3 to position {}...", position);

    // Enable pulse engine before moving
    device.enable_pulse_engine(true)?;

    // Check axis state before moving
    let axis_state_0 = device.get_axis_state(0)?;
    println!("Axis 1 state before move: {:?}", axis_state_0);
    println!(
        "Axis 1 enabled: {}",
        device.pulse_engine_v2.is_axis_enabled(0)
    );

    let current_pos_0 = device.get_axis_position(0)?;
    println!("Axis 1 current position: {}", current_pos_0);

    let axis_state_1 = device.get_axis_state(1)?;
    println!("Axis 2 state before move: {:?}", axis_state_1);
    println!(
        "Axis 2 enabled: {}",
        device.pulse_engine_v2.is_axis_enabled(1)
    );

    let current_pos_1 = device.get_axis_position(1)?;
    println!("Axis 2 current position: {}", current_pos_1);

    // Check axis 3 (index 2) - the one we're actually configuring
    let axis_state_2 = device.get_axis_state(2)?;
    println!("Axis 3 state before move: {:?}", axis_state_2);
    println!(
        "Axis 3 enabled: {}",
        device.pulse_engine_v2.is_axis_enabled(2)
    );

    let current_pos_2 = device.get_axis_position(2)?;
    println!("Axis 3 current position: {}", current_pos_2);

    // Use the existing move_axis_to_position method
    device.move_axis_to_position(2, position, 50.0)?; // 50% speed
    println!("✓ Move command sent");

    // Check state after move command
    let axis_state_after = device.get_axis_state(2)?;
    println!("Axis 3 state after move: {:?}", axis_state_after);

    Ok(())
}
