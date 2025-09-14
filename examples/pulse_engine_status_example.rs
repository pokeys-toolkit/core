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
    println!(
        "  State: {} ({})",
        device.pulse_engine_v2.pulse_engine_state,
        match device.pulse_engine_v2.pulse_engine_state {
            0 => "Stopped",
            1 => "Running",
            _ => "Unknown",
        }
    );
    println!(
        "  Configured axes: {}",
        device.pulse_engine_v2.info.nr_of_axes
    );
    println!("  Enabled: {}", device.pulse_engine_v2.pulse_engine_enabled);
    println!(
        "  Activated: {}",
        device.pulse_engine_v2.pulse_engine_activated
    );

    Ok(())
}
