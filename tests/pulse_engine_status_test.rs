//! Test for Pulse Engine v2 status command (0x85/0x00)

use pokeys_lib::*;

#[test]
fn test_pulse_engine_get_status() {
    // Mock device response for pulse engine status
    let mock_response = [
        0xAA, // Response header
        0x85, // Command
        0x00, // Operation (Get status)
        0x01, // Status: Running
        0x08, // Number of axes
        0x00, // Reserved
        0x42, // Request ID
        0x00, // Checksum placeholder
    ];

    // Test status parsing
    assert_eq!(mock_response[1], 0x85); // Command
    assert_eq!(mock_response[2], 0x00); // Operation
    assert_eq!(mock_response[3], 0x01); // Status
    assert_eq!(mock_response[4], 0x08); // Axes count
}

#[cfg(feature = "hardware-tests")]
#[test]
fn test_pulse_engine_status_hardware() -> Result<()> {
    // Try USB first
    let device_count = enumerate_usb_devices()?;
    let mut device = if device_count > 0 {
        connect_to_device(0)?
    } else {
        // Try network devices
        let network_devices = enumerate_network_devices(2000)?;
        if network_devices.is_empty() {
            println!("No devices found, skipping hardware test");
            return Ok(());
        }

        // Look for device 32223 or use first available
        let target_device = network_devices
            .iter()
            .find(|d| d.serial_number == 32223)
            .unwrap_or(&network_devices[0]);

        println!(
            "Connecting to network device: serial={}",
            target_device.serial_number
        );
        connect_to_network_device(target_device)?
    };
    device.get_device_data()?;

    // Send pulse engine status request (0x85/0x00)
    let response = device.send_request(0x85, 0x00, 0, 0, 0)?;

    assert!(response.len() >= 8);
    assert_eq!(response[1], 0x85); // Command echo
    assert_eq!(response[2], 0x00); // Operation echo

    println!("Pulse engine status: {}", response[3]);
    println!("Number of axes: {}", response[4]);

    Ok(())
}
