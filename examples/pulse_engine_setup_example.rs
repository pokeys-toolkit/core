use pokeys_lib::*;

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

    let config = PulseEngineConfig::three_channel_internal(3, false).build();
    device.setup_pulse_engine(&config)?;

    device.enable_pulse_engine(true)?;

    // Verify configuration
    device.get_pulse_engine_status()?;
    println!(
        "✓ Pulse engine configured: {} axes, generator type 0x{:02X}",
        device.pulse_engine_v2.info.nr_of_axes, device.pulse_engine_v2.pulse_generator_type
    );

    Ok(())
}
