//! Network Configuration Example (PoKeys57E — network)
//!
//! Reads the current network configuration then walks through setting:
//!   - static IP address, subnet mask, gateway, TCP timeout
//!   - DHCP mode
//!   - additional network options (discovery, auto-config, UDP config)
//!
//! Each step reads the configuration back from the device to confirm the
//! change was applied. The example does NOT restore the original settings —
//! run it against a test device.
//!
//! Usage:
//!   cargo run --example network_config
//!   cargo run --example network_config -- 32218   # target a specific serial

use pokeys_lib::{NetworkDeviceConfig, Result, enumerate_network_devices};

fn fmt_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn fmt_options(opts: u8) -> String {
    let discovery = if opts & 0x01 != 0 {
        "disabled"
    } else {
        "enabled"
    };
    let auto_cfg = if opts & 0x02 != 0 {
        "disabled"
    } else {
        "enabled"
    };
    let udp_cfg = if opts & 0x04 != 0 {
        "disabled"
    } else {
        "enabled"
    };
    format!("discovery={discovery}, auto-config={auto_cfg}, udp-config={udp_cfg}")
}

fn main() -> Result<()> {
    let target_serial: Option<u32> = std::env::args().nth(1).and_then(|s| s.parse().ok());

    // Discover
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
            fmt_ip(d.ip_address),
            d.firmware_version_major,
            d.firmware_version_minor,
        );
    }

    let target = match target_serial {
        Some(serial) => devices
            .iter()
            .find(|d| d.serial_number == serial)
            .ok_or_else(|| {
                eprintln!("Device with serial {serial} not found.");
                pokeys_lib::PoKeysError::NotConnected
            })?,
        None => &devices[0],
    };

    println!(
        "\nUsing device serial={} ip={}",
        target.serial_number,
        fmt_ip(target.ip_address)
    );

    let mut device = pokeys_lib::connect_to_device_with_serial(target.serial_number, true, 3000)?;
    device.get_device_data()?;

    // ── Step 1: read current config ──────────────────────────────────────────
    println!("\n── Current network configuration ──────────────────────────────");
    let (_, cfg) = device.get_network_configuration(3000)?;
    println!(
        "  DHCP:          {}",
        if cfg.dhcp_enabled() { "on" } else { "off" }
    );
    println!("  Setup IP:      {}", fmt_ip(cfg.ip_address_setup));
    println!("  Current IP:    {}", fmt_ip(cfg.ip_address_current));
    println!("  Subnet mask:   {}", fmt_ip(cfg.subnet_mask));
    println!("  Gateway:       {}", fmt_ip(cfg.gateway_ip));
    println!("  TCP timeout:   {}ms", cfg.tcp_timeout);
    println!(
        "  Options (raw): {:#04x} ({})",
        cfg.additional_network_options,
        fmt_options(cfg.additional_network_options)
    );

    // ── Step 2: set static IP ────────────────────────────────────────────────
    println!("\n── Step 2: set static IP 10.0.1.103 ───────────────────────────");
    let mut static_cfg = NetworkDeviceConfig::new();
    static_cfg.set_dhcp(false);
    static_cfg.set_ip_address([10, 0, 1, 103]);
    static_cfg.set_subnet_mask([255, 255, 255, 0]);
    static_cfg.set_default_gateway([10, 0, 1, 1]);
    static_cfg.set_tcp_timeout(2000);
    device.set_network_configuration(&static_cfg.device_info)?;
    println!("  Sent. Reading back...");

    let (_, cfg) = device.get_network_configuration(3000)?;
    println!(
        "  DHCP:        {}",
        if cfg.dhcp_enabled() { "on" } else { "off" }
    );
    println!("  Setup IP:    {}", fmt_ip(cfg.ip_address_setup));
    println!("  Subnet mask: {}", fmt_ip(cfg.subnet_mask));
    println!("  Gateway:     {}", fmt_ip(cfg.gateway_ip));
    println!("  TCP timeout: {}ms", cfg.tcp_timeout);

    // ── Step 3: set TCP timeout only ────────────────────────────────────────
    println!("\n── Step 3: change TCP timeout to 5000ms ────────────────────────");
    let mut timeout_cfg = NetworkDeviceConfig::new();
    timeout_cfg.set_dhcp(cfg.dhcp_enabled());
    timeout_cfg.set_ip_address(cfg.ip_address_setup);
    timeout_cfg.set_subnet_mask(cfg.subnet_mask);
    timeout_cfg.set_default_gateway(cfg.gateway_ip);
    timeout_cfg.set_tcp_timeout(5000);
    device.set_network_configuration(&timeout_cfg.device_info)?;
    println!("  Sent. Reading back...");

    let (_, cfg) = device.get_network_configuration(3000)?;
    println!("  TCP timeout: {}ms", cfg.tcp_timeout);

    // ── Step 4: network options ──────────────────────────────────────────────
    println!("\n── Step 4: disable UDP config, keep discovery + auto-config ────");
    let mut opts_cfg = NetworkDeviceConfig::new();
    opts_cfg.set_dhcp(cfg.dhcp_enabled());
    opts_cfg.set_ip_address(cfg.ip_address_setup);
    opts_cfg.set_subnet_mask(cfg.subnet_mask);
    opts_cfg.set_default_gateway(cfg.gateway_ip);
    opts_cfg.set_tcp_timeout(cfg.tcp_timeout);
    opts_cfg.set_network_options(false, false, true); // only disable UDP config
    device.set_network_configuration(&opts_cfg.device_info)?;
    println!("  Sent. Reading back...");

    let (_, cfg) = device.get_network_configuration(3000)?;
    println!(
        "  Options (raw): {:#04x} ({})",
        cfg.additional_network_options,
        fmt_options(cfg.additional_network_options)
    );

    // ── Step 5: enable DHCP ──────────────────────────────────────────────────
    println!("\n── Step 5: enable DHCP ─────────────────────────────────────────");
    let mut dhcp_cfg = NetworkDeviceConfig::new();
    dhcp_cfg.set_dhcp(true);
    device.set_network_configuration(&dhcp_cfg.device_info)?;
    println!("  Sent. Reading back...");

    let (_, cfg) = device.get_network_configuration(3000)?;
    println!("  DHCP: {}", if cfg.dhcp_enabled() { "on" } else { "off" });

    println!("\nDone. Reboot the device for any IP/DHCP changes to take effect.");
    Ok(())
}
