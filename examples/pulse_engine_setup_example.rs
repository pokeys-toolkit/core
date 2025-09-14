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

    // Setup pulse engine with current configuration
    println!("\nSending setup command (0x85/0x01)...");
    device.setup_pulse_engine()?;
    println!("✓ Setup command sent successfully");

    Ok(())
}
