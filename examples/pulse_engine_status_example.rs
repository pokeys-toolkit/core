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

    // Get pulse engine status (0x85/0x00)
    let response = device.send_request(0x85, 0x00, 0, 0, 0)?;

    let status = response[3];
    let axes_count = response[4];

    println!("\nPulse Engine Status:");
    println!(
        "  Status: {} ({})",
        status,
        match status {
            0 => "Stopped",
            1 => "Running",
            _ => "Unknown",
        }
    );
    println!("  Configured axes: {}", axes_count);

    Ok(())
}
