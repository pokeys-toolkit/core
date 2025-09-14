use pokeys_lib::*;

fn main() -> Result<()> {
    println!("PoKeys Pulse Engine Status Example");
    println!("==================================");

    // Enumerate network devices
    let network_devices = enumerate_network_devices(2000)?;
    if network_devices.is_empty() {
        println!("No network devices found");
        return Ok(());
    }

    // Find device 32223
    let target_device = network_devices
        .iter()
        .find(|d| d.serial_number == 32223)
        .ok_or_else(|| PoKeysError::Parameter("Device 32223 not found".to_string()))?;

    println!(
        "Connecting to device 32223 at {:?}",
        target_device.ip_address
    );

    // Connect to device
    let mut device = connect_to_network_device(target_device)?;
    device.get_device_data()?;

    println!("✓ Connected to device {}", target_device.serial_number);

    // Get pulse engine status using PulseEngineV2
    device.get_pulse_engine_status()?;

    println!("\nPulse Engine Status (Complete):");
    let pe = &device.pulse_engine_v2;

    // Basic status
    println!("  State: {} ({:?})", pe.pulse_engine_state, pe.get_state());
    println!("  Enabled axes: {}", pe.info.nr_of_axes);
    println!("  Activated: {}", pe.pulse_engine_activated);
    println!("  Charge pump: {}", pe.charge_pump_enabled);

    // Generator info
    println!("  Generator type: 0x{:02X}", pe.pulse_generator_type);
    println!("  Max frequency: {} kHz", pe.info.max_pulse_frequency);
    println!("  Buffer depth: {}", pe.info.buffer_depth);
    println!("  Slot timing: {} (100us steps)", pe.info.slot_timing);

    // Status flags
    println!("  Soft limit status: 0x{:02X}", pe.soft_limit_status);
    println!(
        "  Axis enabled states: 0x{:02X}",
        pe.axis_enabled_states_mask
    );
    println!("  Limit override: 0x{:02X}", pe.limit_override);
    println!("  Limit+ status: 0x{:02X}", pe.limit_status_p);
    println!("  Limit- status: 0x{:02X}", pe.limit_status_n);
    println!("  Home status: 0x{:02X}", pe.home_status);
    println!(
        "  Emergency switch polarity: {}",
        pe.emergency_switch_polarity
    );
    println!("  Error input status: 0x{:02X}", pe.error_input_status);
    println!("  Misc input status: 0x{:02X}", pe.misc_input_status);

    // Axes status
    println!("\nAxes Status:");
    for i in 0..pe.info.nr_of_axes as usize {
        println!(
            "  Axis {}: status=0x{:02X}, position={}",
            i, pe.axes_state[i], pe.current_position[i]
        );
    }

    Ok(())
}
