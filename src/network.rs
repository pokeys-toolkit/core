//! Network device support and discovery

use crate::communication::NetworkInterface;
use crate::error::{PoKeysError, Result};
use crate::types::{NetworkDeviceInfo, NetworkDeviceSummary};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

/// UDP network interface implementation
pub struct UdpNetworkInterface {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl UdpNetworkInterface {
    pub fn new(remote_ip: [u8; 4], remote_port: u16) -> Result<Self> {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        let socket = UdpSocket::bind(local_addr)?;

        let remote_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(
                remote_ip[0],
                remote_ip[1],
                remote_ip[2],
                remote_ip[3],
            )),
            remote_port,
        );

        Ok(Self {
            socket,
            remote_addr,
        })
    }
}

impl NetworkInterface for UdpNetworkInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize> {
        self.socket
            .send_to(data, self.remote_addr)
            .map_err(PoKeysError::Io)
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let (bytes_received, _) = self.socket.recv_from(buffer)?;
        Ok(bytes_received)
    }

    fn receive_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize> {
        self.socket.set_read_timeout(Some(timeout))?;
        let result = self.receive(buffer);
        self.socket.set_read_timeout(None)?;
        result
    }
}

/// TCP network interface implementation
pub struct TcpNetworkInterface {
    stream: TcpStream,
}

impl TcpNetworkInterface {
    pub fn new(remote_ip: [u8; 4], remote_port: u16) -> Result<Self> {
        let remote_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(
                remote_ip[0],
                remote_ip[1],
                remote_ip[2],
                remote_ip[3],
            )),
            remote_port,
        );

        let stream = TcpStream::connect(remote_addr)?;

        Ok(Self { stream })
    }
}

impl NetworkInterface for TcpNetworkInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize> {
        self.stream.write(data).map_err(PoKeysError::Io)
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.stream.read(buffer).map_err(PoKeysError::Io)
    }

    fn receive_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize> {
        self.stream.set_read_timeout(Some(timeout))?;
        let result = self.receive(buffer);
        self.stream.set_read_timeout(None)?;
        result
    }
}

/// Network device discovery
pub struct NetworkDiscovery {
    socket: UdpSocket,
}

impl NetworkDiscovery {
    pub fn new() -> Result<Self> {
        // Bind to any available port for sending
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        let socket = UdpSocket::bind(local_addr)?;
        socket.set_broadcast(true)?;

        Ok(Self { socket })
    }

    /// Discover PoKeys devices on the network
    pub fn discover_devices(&self, timeout_ms: u32) -> Result<Vec<NetworkDeviceSummary>> {
        let mut devices = Vec::new();
        let mut seen_serials = HashSet::new();

        // Get broadcast addresses to try
        let broadcast_addresses = self.get_broadcast_addresses()?;

        // Send discovery packets to all broadcast addresses
        let discovery_packet = self.create_discovery_packet();

        for &broadcast_addr in &broadcast_addresses {
            let addr = SocketAddr::new(IpAddr::V4(broadcast_addr), 20055);
            log::debug!("Sending discovery packet to {addr}");

            if let Err(e) = self.socket.send_to(&discovery_packet, addr) {
                log::warn!("Failed to send discovery packet to {addr}: {e}");
                continue;
            }
        }

        // Also send to general broadcast
        let general_broadcast =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 20055);
        if let Err(e) = self.socket.send_to(&discovery_packet, general_broadcast) {
            log::warn!("Failed to send general broadcast: {e}");
        }

        // Listen for responses
        let start_time = Instant::now();
        let timeout = Duration::from_millis(timeout_ms as u64);

        // Set a short read timeout to allow checking for overall timeout
        self.socket
            .set_read_timeout(Some(Duration::from_millis(100)))?;

        while start_time.elapsed() < timeout {
            let mut buffer = [0u8; 1024];
            match self.socket.recv_from(&mut buffer) {
                Ok((bytes_received, sender_addr)) => {
                    log::debug!("Received {bytes_received} bytes from {sender_addr}");

                    if let Some(device) =
                        self.parse_discovery_response(&buffer[..bytes_received], sender_addr)
                    {
                        // Avoid duplicate devices (same serial number)
                        if seen_serials.insert(device.serial_number) {
                            log::debug!(
                                "Discovered PoKeys device: Serial {}, IP {}.{}.{}.{}, FW {}.{}",
                                device.serial_number,
                                device.ip_address[0],
                                device.ip_address[1],
                                device.ip_address[2],
                                device.ip_address[3],
                                device.firmware_version_major,
                                device.firmware_version_minor
                            );
                            devices.push(device);
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // Timeout, continue listening
                    continue;
                }
                Err(e) => {
                    log::warn!("Error receiving discovery response: {e}");
                    continue;
                }
            }
        }

        log::info!(
            "Network discovery completed, found {} devices",
            devices.len()
        );
        Ok(devices)
    }

    /// Get broadcast addresses for all network interfaces
    fn get_broadcast_addresses(&self) -> Result<Vec<Ipv4Addr>> {
        let mut addresses = Vec::new();

        // Add common broadcast addresses
        addresses.push(Ipv4Addr::new(255, 255, 255, 255)); // General broadcast
        addresses.push(Ipv4Addr::new(192, 168, 1, 255)); // Common home network
        addresses.push(Ipv4Addr::new(192, 168, 0, 255)); // Common home network
        addresses.push(Ipv4Addr::new(10, 0, 1, 255)); // Common corporate network
        addresses.push(Ipv4Addr::new(172, 16, 0, 255)); // Common corporate network

        // TODO: In a more complete implementation, we would enumerate actual network interfaces
        // and calculate their broadcast addresses. For now, we use common ones.

        Ok(addresses)
    }

    /// Search for specific device by serial number
    pub fn search_device(
        &self,
        serial_number: u32,
        timeout_ms: u32,
    ) -> Result<Option<NetworkDeviceSummary>> {
        let devices = self.discover_devices(timeout_ms)?;

        for device in devices {
            if device.serial_number == serial_number {
                return Ok(Some(device));
            }
        }

        Ok(None)
    }

    fn create_discovery_packet(&self) -> Vec<u8> {
        // PoKeys network discovery uses an empty UDP packet
        // The presence of any UDP packet on port 20055 triggers the device to respond
        Vec::new()
    }

    fn parse_discovery_response(
        &self,
        data: &[u8],
        sender_addr: SocketAddr,
    ) -> Option<NetworkDeviceSummary> {
        // PoKeys devices respond with either 14 bytes (older devices) or 19 bytes (58 series)
        if data.len() != 14 && data.len() != 19 {
            return None;
        }

        let _sender_ip = match sender_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.octets(),
            _ => return None,
        };

        if data.len() == 14 {
            // Older device format (14 bytes)
            // Byte 0: User ID
            // Bytes 1-2: Serial number (16-bit, big-endian)
            // Bytes 3-4: Firmware version (major, minor)
            // Bytes 5-8: IP address (from device response, not sender)
            // Byte 9: DHCP flag
            // Bytes 10-13: Host IP address

            let user_id = data[0];
            let serial_number = ((data[1] as u32) << 8) | (data[2] as u32);

            // Decode firmware version: v(1+[bits 4-7]).(bits [0-3])
            let firmware_version_encoded = data[3];
            let firmware_revision = data[4]; // This might be revision or minor version

            let major_bits = (firmware_version_encoded >> 4) & 0x0F; // Extract bits 4-7
            let minor_bits = firmware_version_encoded & 0x0F; // Extract bits 0-3
            let decoded_major = 1 + major_bits;
            let decoded_minor = minor_bits;

            let device_ip = [data[5], data[6], data[7], data[8]];
            let dhcp = data[9];
            let host_ip = [data[10], data[11], data[12], data[13]];

            Some(NetworkDeviceSummary {
                serial_number,
                ip_address: device_ip,
                host_ip,
                firmware_version_major: decoded_major,
                firmware_version_minor: decoded_minor,
                firmware_revision,
                user_id,
                dhcp,
                hw_type: 0, // Not available in 14-byte format
                use_udp: 1, // Assume UDP for older devices
            })
        } else {
            // 58 series device format (19 bytes)
            // Byte 0: User ID
            // Bytes 1-2: (unused in serial parsing)
            // Bytes 3-4: Firmware version (encoded major, revision/minor)
            // Bytes 5-8: IP address (from device response, not sender)
            // Byte 9: DHCP flag
            // Bytes 10-13: Host IP address
            // Bytes 14-17: Serial number (32-bit, little-endian)
            // Byte 18: Hardware type

            let user_id = data[0];

            // Decode firmware version: v(1+[bits 4-7]).(bits [0-3])
            let firmware_version_encoded = data[3];
            let firmware_revision = data[4]; // This might be revision or minor version

            let major_bits = (firmware_version_encoded >> 4) & 0x0F; // Extract bits 4-7
            let minor_bits = firmware_version_encoded & 0x0F; // Extract bits 0-3
            let decoded_major = 1 + major_bits;
            let decoded_minor = minor_bits;

            let device_ip = [data[5], data[6], data[7], data[8]];
            let dhcp = data[9];
            let host_ip = [data[10], data[11], data[12], data[13]];
            let serial_number = ((data[17] as u32) << 24)
                | ((data[16] as u32) << 16)
                | ((data[15] as u32) << 8)
                | (data[14] as u32);
            let hw_type = data[18];

            Some(NetworkDeviceSummary {
                serial_number,
                ip_address: device_ip,
                host_ip,
                firmware_version_major: decoded_major,
                firmware_version_minor: decoded_minor,
                firmware_revision,
                user_id,
                dhcp,
                hw_type,
                use_udp: 1, // Default to UDP, could be determined by device type
            })
        }
    }
}

/// Network device configuration
pub struct NetworkDeviceConfig {
    pub device_info: NetworkDeviceInfo,
}

impl Default for NetworkDeviceConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDeviceConfig {
    pub fn new() -> Self {
        Self {
            device_info: NetworkDeviceInfo {
                ip_address_current: [0, 0, 0, 0],
                ip_address_setup: [0, 0, 0, 0],
                subnet_mask: [255, 255, 255, 0],
                gateway_ip: [0, 0, 0, 0],
                tcp_timeout: 1000,
                additional_network_options: 0xA0,
                dhcp: 0,
            },
        }
    }

    /// Configure device IP address
    pub fn set_ip_address(&mut self, ip: [u8; 4]) {
        self.device_info.ip_address_setup = ip;
    }

    /// Configure subnet mask
    pub fn set_subnet_mask(&mut self, mask: [u8; 4]) {
        self.device_info.subnet_mask = mask;
    }

    /// Configure default gateway
    pub fn set_default_gateway(&mut self, gateway: [u8; 4]) {
        self.device_info.gateway_ip = gateway;
    }

    /// Enable/disable DHCP
    pub fn set_dhcp(&mut self, enable: bool) {
        self.device_info.dhcp = if enable { 1 } else { 0 };
    }

    /// Set TCP timeout
    pub fn set_tcp_timeout(&mut self, timeout_ms: u16) {
        self.device_info.tcp_timeout = timeout_ms;
    }

    /// Configure network options
    pub fn set_network_options(
        &mut self,
        disable_discovery: bool,
        disable_auto_config: bool,
        disable_udp_config: bool,
    ) {
        let mut options = 0xA0u8; // Base value

        if disable_discovery {
            options |= 0x01;
        }
        if disable_auto_config {
            options |= 0x02;
        }
        if disable_udp_config {
            options |= 0x04;
        }

        self.device_info.additional_network_options = options;
    }
}

/// Network utilities
pub mod network_utils {
    use super::*;

    /// Convert IP address from bytes to string
    pub fn ip_to_string(ip: [u8; 4]) -> String {
        format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
    }

    /// Convert IP address from string to bytes
    pub fn string_to_ip(ip_str: &str) -> Result<[u8; 4]> {
        let parts: Vec<&str> = ip_str.split('.').collect();
        if parts.len() != 4 {
            return Err(PoKeysError::Parameter(
                "Invalid IP address format".to_string(),
            ));
        }

        let mut ip = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            ip[i] = part
                .parse::<u8>()
                .map_err(|_| PoKeysError::Parameter("Invalid IP address octet".to_string()))?;
        }

        Ok(ip)
    }

    /// Check if IP address is in the same subnet
    pub fn same_subnet(ip1: [u8; 4], ip2: [u8; 4], subnet_mask: [u8; 4]) -> bool {
        for i in 0..4 {
            if (ip1[i] & subnet_mask[i]) != (ip2[i] & subnet_mask[i]) {
                return false;
            }
        }
        true
    }

    /// Calculate network address
    pub fn network_address(ip: [u8; 4], subnet_mask: [u8; 4]) -> [u8; 4] {
        [
            ip[0] & subnet_mask[0],
            ip[1] & subnet_mask[1],
            ip[2] & subnet_mask[2],
            ip[3] & subnet_mask[3],
        ]
    }

    /// Calculate broadcast address
    pub fn broadcast_address(ip: [u8; 4], subnet_mask: [u8; 4]) -> [u8; 4] {
        [
            ip[0] | (!subnet_mask[0]),
            ip[1] | (!subnet_mask[1]),
            ip[2] | (!subnet_mask[2]),
            ip[3] | (!subnet_mask[3]),
        ]
    }
}

// Convenience functions for network operations

/// Create UDP connection to PoKeys device
pub fn create_udp_connection(device: &NetworkDeviceSummary) -> Result<Box<dyn NetworkInterface>> {
    let interface = UdpNetworkInterface::new(device.ip_address, 20055)?;
    Ok(Box::new(interface))
}

/// Create TCP connection to PoKeys device
pub fn create_tcp_connection(device: &NetworkDeviceSummary) -> Result<Box<dyn NetworkInterface>> {
    let interface = TcpNetworkInterface::new(device.ip_address, 20055)?;
    Ok(Box::new(interface))
}

/// Discover all PoKeys devices on network
pub fn discover_all_devices(timeout_ms: u32) -> Result<Vec<NetworkDeviceSummary>> {
    let discovery = NetworkDiscovery::new()?;
    discovery.discover_devices(timeout_ms)
}

/// Find specific PoKeys device by serial number
pub fn find_device_by_serial(
    serial_number: u32,
    timeout_ms: u32,
) -> Result<Option<NetworkDeviceSummary>> {
    let discovery = NetworkDiscovery::new()?;
    discovery.search_device(serial_number, timeout_ms)
}

#[cfg(test)]
mod tests {
    use super::network_utils::*;
    use super::*;

    #[test]
    fn test_ip_string_conversion() {
        let ip = [192, 168, 1, 100];
        let ip_str = ip_to_string(ip);
        assert_eq!(ip_str, "192.168.1.100");

        let parsed_ip = string_to_ip(&ip_str).unwrap();
        assert_eq!(parsed_ip, ip);
    }

    #[test]
    fn test_invalid_ip_string() {
        assert!(string_to_ip("192.168.1").is_err());
        assert!(string_to_ip("192.168.1.256").is_err());
        assert!(string_to_ip("not.an.ip.address").is_err());
    }

    #[test]
    fn test_subnet_calculations() {
        let ip1 = [192, 168, 1, 100];
        let ip2 = [192, 168, 1, 200];
        let ip3 = [192, 168, 2, 100];
        let subnet_mask = [255, 255, 255, 0];

        assert!(same_subnet(ip1, ip2, subnet_mask));
        assert!(!same_subnet(ip1, ip3, subnet_mask));

        let network = network_address(ip1, subnet_mask);
        assert_eq!(network, [192, 168, 1, 0]);

        let broadcast = broadcast_address(ip1, subnet_mask);
        assert_eq!(broadcast, [192, 168, 1, 255]);
    }

    #[test]
    fn test_network_device_config() {
        let mut config = NetworkDeviceConfig::new();

        config.set_ip_address([192, 168, 1, 100]);
        assert_eq!(config.device_info.ip_address_setup, [192, 168, 1, 100]);

        config.set_dhcp(true);
        assert_eq!(config.device_info.dhcp, 1);

        config.set_dhcp(false);
        assert_eq!(config.device_info.dhcp, 0);

        config.set_tcp_timeout(2000);
        assert_eq!(config.device_info.tcp_timeout, 2000);
    }

    #[test]
    fn test_network_options() {
        let mut config = NetworkDeviceConfig::new();

        config.set_network_options(true, false, false);
        assert_eq!(config.device_info.additional_network_options & 0x01, 0x01);
        assert_eq!(config.device_info.additional_network_options & 0x02, 0x00);

        config.set_network_options(false, true, true);
        assert_eq!(config.device_info.additional_network_options & 0x01, 0x00);
        assert_eq!(config.device_info.additional_network_options & 0x02, 0x02);
        assert_eq!(config.device_info.additional_network_options & 0x04, 0x04);
    }

    #[test]
    fn test_discovery_packet_format() {
        let discovery = NetworkDiscovery::new().unwrap();
        let packet = discovery.create_discovery_packet();

        // PoKeys discovery packet should be empty (0 bytes)
        assert_eq!(packet.len(), 0, "PoKeys discovery packet must be empty");
    }

    #[test]
    fn test_discovery_response_parsing_14_bytes() {
        let discovery = NetworkDiscovery::new().unwrap();

        // Simulate 14-byte response from older device
        let response = [
            0x01, // User ID
            0x12, 0x34, // Serial number (big-endian): 0x1234 = 4660
            0x02, 0x05, // Firmware version: 2.5
            192, 168, 1, 100,  // Device IP: 192.168.1.100
            0x01, // DHCP enabled
            192, 168, 1, 1, // Host IP: 192.168.1.1
        ];

        let sender_addr = "192.168.1.100:20055".parse().unwrap();
        let device = discovery
            .parse_discovery_response(&response, sender_addr)
            .unwrap();

        assert_eq!(device.serial_number, 4660);
        assert_eq!(device.firmware_version_major, 2);
        assert_eq!(device.firmware_version_minor, 5);
        assert_eq!(device.firmware_revision, 5); // From data[4]
        assert_eq!(device.ip_address, [192, 168, 1, 100]);
        assert_eq!(device.dhcp, 1);
        assert_eq!(device.host_ip, [192, 168, 1, 1]);
        assert_eq!(device.hw_type, 0); // Not available in 14-byte format
    }

    #[test]
    fn test_discovery_response_parsing_19_bytes() {
        let discovery = NetworkDiscovery::new().unwrap();

        // Simulate 19-byte response from 58 series device
        let response = [
            0x02, // User ID
            0x00, 0x00, // Unused bytes
            0x03, 0x01, // Firmware version: 3.1
            192, 168, 1, 101,  // Device IP: 192.168.1.101
            0x00, // DHCP disabled
            192, 168, 1, 1, // Host IP: 192.168.1.1
            0x78, 0x56, 0x34, 0x12, // Serial number (little-endian): 0x12345678
            0x58, // Hardware type: 58 series
        ];

        let sender_addr = "192.168.1.101:20055".parse().unwrap();
        let device = discovery
            .parse_discovery_response(&response, sender_addr)
            .unwrap();

        assert_eq!(device.serial_number, 0x12345678);
        assert_eq!(device.firmware_version_major, 3);
        assert_eq!(device.firmware_version_minor, 1);
        assert_eq!(device.firmware_revision, 1); // From data[4]
        assert_eq!(device.ip_address, [192, 168, 1, 101]);
        assert_eq!(device.dhcp, 0);
        assert_eq!(device.host_ip, [192, 168, 1, 1]);
        assert_eq!(device.hw_type, 0x58);
    }

    #[test]
    fn test_discovery_response_invalid_length() {
        let discovery = NetworkDiscovery::new().unwrap();

        // Test invalid response lengths
        let short_response = [0x01, 0x02, 0x03]; // Too short
        let long_response = [0u8; 25]; // Too long
        let sender_addr = "192.168.1.100:20055".parse().unwrap();

        assert!(discovery
            .parse_discovery_response(&short_response, sender_addr)
            .is_none());
        assert!(discovery
            .parse_discovery_response(&long_response, sender_addr)
            .is_none());
    }
}
