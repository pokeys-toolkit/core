//! MAX7219 Console Test - Rust Port of C# Implementation
//!
//! This example replicates the functionality from the C# MAX7219 implementation
//! found in /Users/marcteichtahl/Downloads/MAX7219/MAX7219/
//!
//! Features:
//! - Multiple MAX7219 devices in daisy chain (up to 8)
//! - Refresh tracking system (only sends changed data)
//! - Individual device control (shutdown, intensity, decoder, digits)
//! - Console-based operation with interactive menu

use pokeys_lib::*;
use std::io::{self, Write};

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

// MAX7219 Register Addresses (matching C# implementation)
const REG_DIGIT_0: u8 = 0x01;
const REG_DECODE_MODE: u8 = 0x09;
const REG_INTENSITY: u8 = 0x0A;
const REG_SCAN_LIMIT: u8 = 0x0B;
const REG_SHUTDOWN: u8 = 0x0C;
const REG_NO_OP: u8 = 0x00;

/// Single MAX7219 device controller (matches cMAX7219 class)
#[derive(Debug, Clone)]
pub struct Max7219Device {
    pub shutdown: u8,          // 0 = shutdown, 1 = normal
    pub digit_decoder: u8,     // BCD decoder configuration
    pub digits_count: u8,      // Number of digits (scan limit)
    pub intensity: u8,         // Brightness (0-15)
    pub digits_data: [u8; 8],  // Data for each digit
    needs_refresh: [bool; 12], // Tracks which registers need updating
}

impl Default for Max7219Device {
    fn default() -> Self {
        Self::new()
    }
}

impl Max7219Device {
    pub fn new() -> Self {
        Self {
            shutdown: 1,
            digit_decoder: 0,
            digits_count: 8,
            intensity: 15,
            digits_data: [0; 8],
            needs_refresh: [true; 12], // Initially all need refresh
        }
    }

    /// Get the next data that needs to be sent (matches GetDataToOutput)
    pub fn get_data_to_output(&mut self) -> [u8; 2] {
        // Check in priority order (matches C# implementation)
        if self.needs_refresh[0] {
            self.needs_refresh[0] = false;
            return [REG_SHUTDOWN, self.shutdown];
        }
        if self.needs_refresh[1] {
            self.needs_refresh[1] = false;
            return [REG_DECODE_MODE, self.digit_decoder];
        }
        if self.needs_refresh[2] {
            self.needs_refresh[2] = false;
            return [REG_SCAN_LIMIT, self.digits_count];
        }
        if self.needs_refresh[3] {
            self.needs_refresh[3] = false;
            return [REG_INTENSITY, self.intensity];
        }

        // Check digit data
        for i in 0..8 {
            if self.needs_refresh[4 + i] {
                self.needs_refresh[4 + i] = false;
                return [REG_DIGIT_0 + i as u8, self.digits_data[i]];
            }
        }

        // No operation needed
        [REG_NO_OP, 0x00]
    }

    /// Check if any data needs refreshing (matches DataToRefresh)
    pub fn data_to_refresh(&self) -> bool {
        self.needs_refresh.iter().any(|&x| x)
    }

    /// Set shutdown mode (matches SetShutdown)
    pub fn set_shutdown(&mut self, value: u8) {
        if self.shutdown != value {
            self.needs_refresh[0] = true;
            self.shutdown = value;
        }
    }

    /// Set digit decoder (matches SetDigitDecoder)
    pub fn set_digit_decoder(&mut self, value: u8) {
        if self.digit_decoder != value {
            self.needs_refresh[1] = true;
            self.digit_decoder = value;
        }
    }

    /// Set digits count (matches SetDigitsCount)
    pub fn set_digits_count(&mut self, value: u8) {
        if self.digits_count != value {
            self.needs_refresh[2] = true;
            self.digits_count = value;
        }
    }

    /// Set intensity (matches SetIntensity)
    pub fn set_intensity(&mut self, value: u8) {
        if self.intensity != value {
            self.needs_refresh[3] = true;
            self.intensity = value;
        }
    }

    /// Set digit data (matches SetDigit)
    pub fn set_digit(&mut self, digit: u8, value: u8) {
        if digit >= 8 {
            return;
        }
        if self.digits_data[digit as usize] != value {
            self.needs_refresh[4 + digit as usize] = true;
            self.digits_data[digit as usize] = value;
        }
    }
}

/// MAX7219 driver for multiple devices (matches cMAX7219driver class)
pub struct Max7219Driver {
    device: PoKeysDevice,
    pin_cs: u8,
    devices_in_string: usize,
    pub drivers: Vec<Max7219Device>,
}

impl Max7219Driver {
    /// Create new driver (matches C# constructor)
    pub fn new(devices_in_string: usize, mut device: PoKeysDevice, pin_cs: u8) -> Result<Self> {
        // IMPORTANT: Configure CS pin as digital output and set HIGH (idle state)
        println!("🔧 Configuring CS pin {pin_cs} as digital output...");
        device.set_pin_function(pin_cs.into(), PinFunction::DigitalOutput)?;

        println!("🔧 Setting CS pin HIGH (idle state for MAX7219)...");
        device.set_digital_output(pin_cs.into(), true)?; // HIGH = idle state
        std::thread::sleep(std::time::Duration::from_millis(10)); // Let pin settle

        // Initialize SPI - matches C# SPIconfigure(100, 0)
        // 100 prescaler = 250 kHz (25 MHz / 100), clock polarity = low, clock phase = first change
        device.spi_configure(100, 0)?;

        let mut drivers = Vec::new();
        for _ in 0..devices_in_string {
            drivers.push(Max7219Device::new());
        }

        Ok(Self {
            device,
            pin_cs,
            devices_in_string: devices_in_string.min(8), // Max 8 devices
            drivers,
        })
    }

    /// Send SPI data (matches SPI method)
    fn spi(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        // Debug output (matches commented code in C#)
        print!("SPI TX: ");
        for byte in data {
            print!("{byte:02X} ");
        }
        println!();

        // Send data via SPI
        self.device.spi_write(data, self.pin_cs)?;

        // Read back data (matches SPIRead call)
        let read_data = self.device.spi_read(data.len() as u8)?;

        print!("SPI RX: ");
        for byte in &read_data {
            print!("{byte:02X} ");
        }
        println!();

        Ok(read_data)
    }

    /// Refresh display (matches RefreshDisplay method)
    pub fn refresh_display(&mut self) -> Result<()> {
        let mut data_to_send = vec![0u8; self.devices_in_string * 2];
        let mut refresh_required = true;

        while refresh_required {
            refresh_required = false;

            // Get data for each MAX7219 in the string
            for i in 0..self.devices_in_string {
                let device_data = self.drivers[i].get_data_to_output();
                data_to_send[i * 2] = device_data[0];
                data_to_send[i * 2 + 1] = device_data[1];

                // Check if any device still needs refresh
                refresh_required = refresh_required || self.drivers[i].data_to_refresh();
            }

            // Send data via SPI
            self.spi(&data_to_send)?;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    println!("MAX7219 Console Test - Rust Port");
    println!("=================================");
    println!("Based on C# implementation from Downloads/MAX7219");
    println!("Using network device with serial: 32218");
    println!();

    // Connect to network device with serial 32218
    println!("🔍 Discovering network devices...");
    let network_devices = enumerate_network_devices(3000)?;
    if network_devices.is_empty() {
        println!("❌ No network devices found!");
        return Ok(());
    }

    println!("✅ Found {} network device(s)", network_devices.len());

    // Find device with serial 32218
    let target_device = network_devices
        .iter()
        .find(|dev| dev.serial_number == 32218);
    if target_device.is_none() {
        println!("❌ Network device with serial 32218 not found!");
        println!("Available devices:");
        for dev in &network_devices {
            println!(
                "   Serial: {}, IP: {}",
                dev.serial_number,
                format_ip(dev.ip_address)
            );
        }
        return Ok(());
    }

    let device_info = target_device.unwrap();
    println!(
        "✅ Found target device - Serial: {}, IP: {}",
        device_info.serial_number,
        format_ip(device_info.ip_address)
    );

    let device = connect_to_device_with_serial(32218, true, 3000)?;
    println!(
        "✅ Connected to network device: {}",
        device.device_data.serial_number
    );

    // Initialize MAX7219 display driver (matches C# constructor call)
    // Using 8 devices, pin 8 as CS (matches C# example)
    let mut display_driver = Max7219Driver::new(8, device, 8)?;
    println!("✅ Initialized MAX7219 driver with 8 devices, CS pin 8");

    loop {
        println!("\n🎮 MAX7219 Console Test Menu:");
        println!("1. Run C# Example Pattern (Form1.button1_Click equivalent)");
        println!("2. Set Individual Device Parameters");
        println!("3. Display Test Patterns");
        println!("4. Brightness Sweep");
        println!("5. Shutdown All Devices");
        println!("6. Wake Up All Devices");
        println!("7. Exit");
        print!("\nSelect option (1-7): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => run_csharp_example(&mut display_driver)?,
            "2" => set_individual_parameters(&mut display_driver)?,
            "3" => display_test_patterns(&mut display_driver)?,
            "4" => brightness_sweep(&mut display_driver)?,
            "5" => shutdown_all_devices(&mut display_driver)?,
            "6" => wake_up_all_devices(&mut display_driver)?,
            "7" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option. Please select 1-7."),
        }
    }

    Ok(())
}

/// Run the exact pattern from the C# Form1.button1_Click method
fn run_csharp_example(driver: &mut Max7219Driver) -> Result<()> {
    println!("\n🔄 Running C# Example Pattern...");
    println!("This replicates the exact code from Form1.button1_Click");

    // Set data for all 8 displays (matches C# for loop)
    for i in 0..8 {
        println!("   Configuring device {i}...");

        // Set digit data: each digit gets (1 << digit_position)
        for d in 0..8 {
            driver.drivers[i].set_digit(d, 1 << d);
        }

        // Set intensity (matches C# i * 2)
        driver.drivers[i].set_intensity((i * 2) as u8);

        // Set digit decoder: BCD on digits 1 and 5 (matches C# (1 << 1) | (1 << 5))
        driver.drivers[i].set_digit_decoder((1 << 1) | (1 << 5));

        // Set normal mode (matches C# SetShutdown(1))
        driver.drivers[i].set_shutdown(1);
    }

    println!("   Refreshing display...");
    driver.refresh_display()?;

    println!("✅ C# example pattern completed!");
    println!("   Each device should show a different pattern with varying brightness");

    Ok(())
}

/// Set parameters for individual devices
fn set_individual_parameters(driver: &mut Max7219Driver) -> Result<()> {
    println!("\n⚙️  Individual Device Configuration");

    print!("Enter device number (0-{}): ", driver.devices_in_string - 1);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let device_num: usize = input.trim().parse().unwrap_or(0);

    if device_num >= driver.devices_in_string {
        println!("❌ Invalid device number");
        return Ok(());
    }

    println!("Configuring device {device_num}:");

    // Set intensity
    print!("Enter intensity (0-15): ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let intensity: u8 = input.trim().parse().unwrap_or(8);
    driver.drivers[device_num].set_intensity(intensity.min(15));

    // Set digit decoder
    print!("Enter digit decoder (0-255): ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let decoder: u8 = input.trim().parse().unwrap_or(0);
    driver.drivers[device_num].set_digit_decoder(decoder);

    // Set individual digits
    for digit in 0..8 {
        print!("Enter value for digit {digit} (0-255): ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        let value: u8 = input.trim().parse().unwrap_or(0);
        driver.drivers[device_num].set_digit(digit, value);
    }

    driver.refresh_display()?;
    println!("✅ Device {device_num} configured and refreshed!");

    Ok(())
}

/// Display various test patterns
fn display_test_patterns(driver: &mut Max7219Driver) -> Result<()> {
    println!("\n🎯 Test Patterns");

    // Pattern 1: All segments on
    println!("   Pattern 1: All segments on");
    for i in 0..driver.devices_in_string {
        for digit in 0..8 {
            driver.drivers[i].set_digit(digit as u8, 0xFF);
        }
        driver.drivers[i].set_shutdown(1);
    }
    driver.refresh_display()?;
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 2: Checkerboard
    println!("   Pattern 2: Checkerboard");
    for i in 0..driver.devices_in_string {
        for digit in 0..8 {
            let pattern = if (digit + i) % 2 == 0 { 0xAA } else { 0x55 };
            driver.drivers[i].set_digit(digit as u8, pattern);
        }
    }
    driver.refresh_display()?;
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 3: Walking bit
    println!("   Pattern 3: Walking bit");
    for bit in 0..8 {
        for i in 0..driver.devices_in_string {
            for digit in 0..8 {
                driver.drivers[i].set_digit(digit as u8, 1 << bit);
            }
        }
        driver.refresh_display()?;
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    println!("✅ Test patterns completed!");
    Ok(())
}

/// Sweep brightness across all devices
fn brightness_sweep(driver: &mut Max7219Driver) -> Result<()> {
    println!("\n💡 Brightness Sweep");

    // Set a visible pattern first
    for i in 0..driver.devices_in_string {
        for digit in 0..8 {
            driver.drivers[i].set_digit(digit as u8, 0x7E); // Display "0" pattern
        }
        driver.drivers[i].set_digit_decoder(0xFF); // BCD mode
        driver.drivers[i].set_shutdown(1);
    }

    // Sweep brightness
    for brightness in 0..=15 {
        println!("   Brightness: {brightness}");
        for i in 0..driver.devices_in_string {
            driver.drivers[i].set_intensity(brightness);
        }
        driver.refresh_display()?;
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    println!("✅ Brightness sweep completed!");
    Ok(())
}

/// Shutdown all devices
fn shutdown_all_devices(driver: &mut Max7219Driver) -> Result<()> {
    println!("\n🔌 Shutting down all devices...");

    for i in 0..driver.devices_in_string {
        driver.drivers[i].set_shutdown(0); // 0 = shutdown
    }
    driver.refresh_display()?;

    println!("✅ All devices shut down!");
    Ok(())
}

/// Wake up all devices
fn wake_up_all_devices(driver: &mut Max7219Driver) -> Result<()> {
    println!("\n🌅 Waking up all devices...");

    for i in 0..driver.devices_in_string {
        driver.drivers[i].set_shutdown(1); // 1 = normal mode
    }
    driver.refresh_display()?;

    println!("✅ All devices awake!");
    Ok(())
}
