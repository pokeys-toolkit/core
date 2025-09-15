use pokeys_lib::pulse_engine::PulseEnginePowerState;
use pokeys_lib::*;

fn main() -> Result<()> {
    println!("PoKeys Pulse Engine Setup Example");
    println!("=================================");

    // Find device 32223
    let network_devices = enumerate_network_devices(2000)?;
    let target_device = network_devices
        .iter()
        .find(|device| device.serial_number == 32223)
        .ok_or_else(|| PoKeysError::Parameter("Device 32223 not found".to_string()))?;

    println!("Connecting to device {}...", target_device.serial_number);
    let mut device = connect_to_network_device(target_device)?;

    // Get current pulse engine status
    device.get_pulse_engine_status()?;
    println!("Current pulse engine configuration:");
    println!(
        "  Enabled axes: {}",
        device.pulse_engine_v2.pulse_engine_enabled
    );
    println!(
        "  Generator type: 0x{:02X}",
        device.pulse_engine_v2.pulse_generator_type
    );
    println!(
        "  Charge pump: {}",
        device.pulse_engine_v2.charge_pump_enabled
    );

    // Configure for 3-axis internal pulse generator (no step/dir swap)
    println!("\nConfiguring for 3-axis internal pulse generator (no step/dir swap)...");

    // First disable pulse engine
    println!("Disabling pulse engine...");
    device.enable_pulse_engine(false)?;

    // Create configuration for 3-axis internal generator without step/dir swap
    let config = PulseEngineConfig::three_channel_internal(3, false).build();
    println!("Using 3-axis internal configuration:");
    println!("  Axes: {}", config.enabled_axes);
    println!("  Generator type: 0x{:02X}", config.generator_type);
    println!("  Charge pump: {}", config.charge_pump_enabled);
    println!("  Buffer size: {}", config.buffer_size);
    println!("  Emergency polarity: {}", config.emergency_switch_polarity);
    println!(
        "  Power states: 0x{:02X} ({})",
        config.power_states,
        if config.power_states == PulseEnginePowerState::ALL_POWER_ENABLED {
            "All power enabled"
        } else {
            "Custom power states"
        }
    );

    // Setup pulse engine with 3ch internal configuration
    println!("Sending setup command (0x85/0x01)...");
    println!("  Axes: {}", config.enabled_axes);
    println!("  Generator: 0x{:02X}", config.generator_type);
    println!("  Charge pump: {}", config.charge_pump_enabled);
    println!("  Emergency polarity: {}", config.emergency_switch_polarity);
    println!("  Power states: 0x{:02X}", config.power_states);

    device.setup_pulse_engine(&config)?;
    println!("✓ Setup command sent successfully");

    // Re-enable pulse engine
    println!("Enabling pulse engine...");
    device.enable_pulse_engine(true)?;
    println!("✓ Pulse engine enabled");

    // Verify configuration
    device.get_pulse_engine_status()?;
    println!("\nVerified configuration:");
    println!(
        "  Enabled axes: {}",
        device.pulse_engine_v2.pulse_engine_enabled
    );
    println!(
        "  Generator: {} ({})",
        device.pulse_engine_v2.get_generator_type(),
        device.pulse_engine_v2.get_generator_type_description()
    );
    println!(
        "  Raw generator type: 0x{:02X}",
        device.pulse_engine_v2.pulse_generator_type
    );

    println!("\n✓ Pulse engine setup complete");

    Ok(())
}
