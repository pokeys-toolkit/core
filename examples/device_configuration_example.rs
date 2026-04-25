//! Device Configuration Example
//!
//! Demonstrates how to set a device name and configure network settings
//! (static IP or DHCP) on a PoKeys network device.

use pokeys_lib::{NetworkDeviceConfig, Result, enumerate_network_devices};

fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn main() -> Result<()> {
    // Step 1: Discover network devices
    println!("Discovering network devices (3 second timeout)...");
    let devices = enumerate_network_devices(3000)?;

    if devices.is_empty() {
        println!("No network devices found. Connect a PoKeys Ethernet device and retry.");
        return Ok(());
    }

    println!("Found {} device(s):", devices.len());
    for (i, d) in devices.iter().enumerate() {
        println!(
            "  {}. serial={} ip={}",
            i + 1,
            d.serial_number,
            format_ip(d.ip_address)
        );
    }

    let target = &devices[0];
    println!("\nUsing device serial={}", target.serial_number);

    let mut device = pokeys_lib::connect_to_device_with_serial(target.serial_number, true, 3000)?;

    // Step 2: Set device name (auto-saves to non-volatile storage)
    println!("\nSetting device name to \"MyPoKeys57E\"...");
    device.set_device_name("MyPoKeys57E")?;
    println!("Device name saved.");

    // Step 3: Read current network configuration
    println!("\nReading current network configuration...");
    let (_, current) = device.get_network_configuration(3000)?;
    println!(
        "  DHCP: {}",
        if current.dhcp_enabled() { "on" } else { "off" }
    );
    println!("  Setup IP:  {}", format_ip(current.ip_address_setup));
    println!("  Current IP: {}", format_ip(current.ip_address()));
    println!("  Subnet:    {}", format_ip(current.subnet_mask));
    println!("  Gateway:   {}", format_ip(current.gateway()));
    println!("  TCP timeout: {}ms", current.tcp_timeout);

    // Step 4: Apply a static IP configuration
    println!("\nApplying static IP 192.168.1.50 ...");
    let mut static_cfg = NetworkDeviceConfig::new();
    static_cfg.set_dhcp(false);
    static_cfg.set_ip_address([192, 168, 1, 50]);
    static_cfg.set_subnet_mask([255, 255, 255, 0]);
    static_cfg.set_default_gateway([192, 168, 1, 1]);
    static_cfg.set_tcp_timeout(1000);
    device.set_network_configuration(&static_cfg.device_info)?;
    println!("Static IP configuration saved (reboot device to apply).");

    // Step 5: Switch to DHCP
    println!("\nSwitching to DHCP...");
    let mut dhcp_cfg = NetworkDeviceConfig::new();
    dhcp_cfg.set_dhcp(true);
    device.set_network_configuration(&dhcp_cfg.device_info)?;
    println!("DHCP configuration saved (reboot device to apply).");

    println!("\nDone. Reboot the device for network changes to take effect.");
    Ok(())
}
