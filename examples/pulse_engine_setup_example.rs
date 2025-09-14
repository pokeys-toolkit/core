use pokeys_lib::*;

fn main() -> Result<()> {
    println!("PoKeys Pulse Engine Setup Example");
    println!("=================================");

    // Find device 32223
    let network_devices = enumerate_network_devices(2000)?;
    let target_device = network_devices
        .iter()
        .find(|d| d.serial_number == 32223)
        .ok_or_else(|| PoKeysError::Parameter("Device 32223 not found".to_string()))?;

    println!("Connecting to device 32223...");
    let mut device = connect_to_network_device(target_device)?;
    device.get_device_data()?;

    // Get current status first
    device.get_pulse_engine_status()?;
    println!("Current pulse engine configuration:");
    println!("  Enabled axes: {}", device.pulse_engine_v2.info.nr_of_axes);
    println!(
        "  Generator type: 0x{:02X}",
        device.pulse_engine_v2.pulse_generator_type
    );
    println!(
        "  Charge pump: {}",
        device.pulse_engine_v2.charge_pump_enabled
    );

    // Configure for 3-channel internal pulse generator
    println!("\nConfiguring for 3-channel internal pulse generator...");

    // First disable pulse engine
    println!("Disabling pulse engine...");
    device.enable_pulse_engine(false)?;

    // Create configuration for 3-channel internal generator
    let config = PulseEngineConfig::three_channel_internal(3);
    println!("Using 3-channel internal configuration");

    // Alternative: 8-channel external configuration
    // let config = PulseEngineConfig::eight_channel_external(8);
    // println!("Using 8-channel external configuration");

    // Setup pulse engine with 3ch internal configuration
    println!("Sending setup command (0x85/0x01)...");
    println!("  Axes: {}", config.enabled_axes);
    println!("  Generator: 0x{:02X}", config.generator_type);
    println!("  Charge pump: {}", config.charge_pump_enabled);
    println!("  Emergency polarity: {}", config.emergency_switch_polarity);
    println!("  Power states: 0x{:02X}", config.power_states);

    device.setup_pulse_engine(&config)?;
    println!("✓ Setup command sent successfully");

    // Enable pulse engine
    println!("Enabling pulse engine...");
    device.enable_pulse_engine(true)?;
    println!("✓ Pulse engine enabled");

    // Small delay for device to process
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Verify configuration
    device.get_pulse_engine_status()?;
    println!("\nVerified configuration:");
    println!("  Enabled axes: {}", device.pulse_engine_v2.info.nr_of_axes);
    println!(
        "  Generator: {} ({})",
        device.pulse_engine_v2.get_generator_type(),
        device.pulse_engine_v2.get_generator_type_description()
    );

    println!(
        "  Raw generator type: 0x{:02X}",
        device.pulse_engine_v2.pulse_generator_type
    );

    // Show pulse engine status
    println!("\nPulse Engine Status:");
    println!(
        "  State: {} ({:?})",
        device.pulse_engine_v2.pulse_engine_state,
        device.pulse_engine_v2.get_state()
    );
    println!(
        "  Activated: {}",
        device.pulse_engine_v2.pulse_engine_activated
    );
    println!(
        "  Charge pump: {}",
        device.pulse_engine_v2.charge_pump_enabled
    );

    // Show axes status
    println!("\nAxes Status:");
    for i in 0..device.pulse_engine_v2.info.nr_of_axes as usize {
        let axis_state = device.pulse_engine_v2.get_axis_state(i);
        println!(
            "  Axis {}: status=0x{:02X} ({:?}), position={}",
            i,
            device.pulse_engine_v2.axes_state[i],
            axis_state,
            device.pulse_engine_v2.current_position[i]
        );
    }

    // Move axis positions in a loop
    println!("\nMoving axis positions (Ctrl+C to stop)...");
    let mut position = 0i32;
    loop {
        position += 1;

        // Set position for axis 0
        device.set_axis_position(0, position)?;

        // Read back current status to show actual position
        device.get_pulse_engine_status()?;
        let actual_position = device.pulse_engine_v2.current_position[0];

        println!(
            "Set axis 0 position to: {} (actual: {})",
            position, actual_position
        );

        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}
