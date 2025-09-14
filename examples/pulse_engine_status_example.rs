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

    println!("\nPulse Engine Status:");
    let state = device.pulse_engine_v2.pulse_engine_state;
    println!("  State: 0x{:02X}", state);
    println!(
        "    STOPPED: {}",
        if state & 0x01 != 0 { "Yes" } else { "No" }
    );
    println!(
        "    STOP_LIMIT: {}",
        if state & 0x02 != 0 { "Yes" } else { "No" }
    );
    println!(
        "    STOP_EMERGENCY: {}",
        if state & 0x04 != 0 { "Yes" } else { "No" }
    );
    println!("  Enabled axes: {}", device.pulse_engine_v2.info.nr_of_axes);
    println!(
        "  Activated: {}",
        device.pulse_engine_v2.pulse_engine_activated
    );
    println!(
        "  Charge pump: {}",
        device.pulse_engine_v2.charge_pump_enabled
    );
    println!(
        "  Generator type: 0x{:02X}",
        device.pulse_engine_v2.pulse_generator_type
    );
    println!(
        "  Max frequency: {} kHz",
        device.pulse_engine_v2.info.max_pulse_frequency
    );
    println!(
        "  Buffer depth: {}",
        device.pulse_engine_v2.info.buffer_depth
    );

    // Show axis positions if any axes are configured
    if device.pulse_engine_v2.info.nr_of_axes > 0 {
        println!("\nAxis Positions:");
        for i in 0..device.pulse_engine_v2.info.nr_of_axes as usize {
            println!(
                "  Axis {}: {}",
                i, device.pulse_engine_v2.current_position[i]
            );
        }
    }

    Ok(())
}
