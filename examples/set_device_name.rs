//! Set Device Name Example (PoKeys57E — network)
//!
//! Discovers PoKeys57E devices on the network, reads the current name,
//! sets a new one, reads it back to confirm, then restores the original.
//!
//! Usage:
//!   cargo run --example set_device_name
//!   cargo run --example set_device_name -- "MyNewName"
//!   cargo run --example set_device_name -- "MyNewName" 32218

use pokeys_lib::*;

fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn parse_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw)
        .trim_end_matches('\0')
        .to_string()
}

fn main() -> Result<()> {
    let new_name: String = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "PoKeys-Test".to_string());
    let target_serial: Option<u32> = std::env::args().nth(2).and_then(|s| s.parse().ok());

    if new_name.len() > 20 {
        eprintln!(
            "Name '{}' is {} chars — will be truncated to 20: '{}'",
            new_name,
            new_name.len(),
            &new_name[..20]
        );
    }

    // Discover network devices
    println!("Discovering PoKeys57E devices on the network (5s timeout)...");
    let devices = enumerate_network_devices(5000)?;

    if devices.is_empty() {
        eprintln!("No network devices found. Check that:");
        eprintln!("  - The PoKeys57E is powered on and connected to the network");
        eprintln!("  - The device and this machine are on the same subnet");
        eprintln!("  - No firewall is blocking UDP discovery");
        return Ok(());
    }

    println!("Found {} device(s):", devices.len());
    for (i, d) in devices.iter().enumerate() {
        println!(
            "  {}. serial={} ip={} fw={}.{}",
            i + 1,
            d.serial_number,
            format_ip(d.ip_address),
            d.firmware_version_major,
            d.firmware_version_minor,
        );
    }

    // Pick target device
    let target = match target_serial {
        Some(serial) => devices
            .iter()
            .find(|d| d.serial_number == serial)
            .ok_or_else(|| {
                eprintln!("Device with serial {serial} not found.");
                PoKeysError::NotConnected
            })?,
        None => &devices[0],
    };

    println!(
        "\nUsing device serial={} ip={}",
        target.serial_number,
        format_ip(target.ip_address)
    );

    // Connect
    let mut device = connect_to_device_with_serial(target.serial_number, true, 3000)?;
    device.get_device_data()?;

    let original_name = parse_name(&device.device_data.device_name);
    println!("Serial:        {}", device.device_data.serial_number);
    println!("Current name:  '{original_name}'");

    // Set new name
    println!("\nSetting name to '{new_name}'...");
    device.set_device_name(&new_name)?;
    println!("Command sent.");

    // Read back to confirm
    device.read_device_data()?;
    let confirmed_name = parse_name(&device.device_data.device_name);
    println!("Name read back: '{confirmed_name}'");

    let expected = &new_name[..new_name.len().min(20)];
    if confirmed_name == expected {
        println!("Name set successfully.");
    } else {
        eprintln!("WARNING: name mismatch — expected '{expected}', got '{confirmed_name}'");
    }

    Ok(())
}
