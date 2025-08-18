//! MAX7219 Console Demo - No Hardware Required
//!
//! This demonstrates the MAX7219 driver functionality without requiring
//! actual hardware. It shows the SPI commands that would be sent.

use std::io::{self, Write};

// MAX7219 Register Addresses
const REG_DIGIT_0: u8 = 0x01;
const REG_DECODE_MODE: u8 = 0x09;
const REG_INTENSITY: u8 = 0x0A;
const REG_SCAN_LIMIT: u8 = 0x0B;
const REG_SHUTDOWN: u8 = 0x0C;
const REG_NO_OP: u8 = 0x00;

/// Mock MAX7219 device for demonstration
#[derive(Debug, Clone)]
pub struct Max7219Device {
    pub shutdown: u8,
    pub digit_decoder: u8,
    pub digits_count: u8,
    pub intensity: u8,
    pub digits_data: [u8; 8],
    needs_refresh: [bool; 12],
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
            needs_refresh: [true; 12],
        }
    }

    pub fn get_data_to_output(&mut self) -> [u8; 2] {
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

        for i in 0..8 {
            if self.needs_refresh[4 + i] {
                self.needs_refresh[4 + i] = false;
                return [REG_DIGIT_0 + i as u8, self.digits_data[i]];
            }
        }

        [REG_NO_OP, 0x00]
    }

    pub fn data_to_refresh(&self) -> bool {
        self.needs_refresh.iter().any(|&x| x)
    }

    pub fn set_shutdown(&mut self, value: u8) {
        if self.shutdown != value {
            self.needs_refresh[0] = true;
            self.shutdown = value;
        }
    }

    pub fn set_digit_decoder(&mut self, value: u8) {
        if self.digit_decoder != value {
            self.needs_refresh[1] = true;
            self.digit_decoder = value;
        }
    }

    pub fn set_digits_count(&mut self, value: u8) {
        if self.digits_count != value {
            self.needs_refresh[2] = true;
            self.digits_count = value;
        }
    }

    pub fn set_intensity(&mut self, value: u8) {
        if self.intensity != value {
            self.needs_refresh[3] = true;
            self.intensity = value;
        }
    }

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

/// Mock driver for demonstration
pub struct Max7219Driver {
    pin_cs: u8,
    devices_in_string: usize,
    pub drivers: Vec<Max7219Device>,
    spi_transaction_count: usize,
}

impl Max7219Driver {
    pub fn new(devices_in_string: usize, pin_cs: u8) -> Self {
        let mut drivers = Vec::new();
        for _ in 0..devices_in_string {
            drivers.push(Max7219Device::new());
        }

        Self {
            pin_cs,
            devices_in_string: devices_in_string.min(8),
            drivers,
            spi_transaction_count: 0,
        }
    }

    fn mock_spi(&mut self, data: &[u8]) {
        self.spi_transaction_count += 1;
        println!(
            "📡 SPI Transaction #{} (CS Pin {}): {:02X?}",
            self.spi_transaction_count, self.pin_cs, data
        );

        // Decode the commands for better understanding
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                let register = chunk[0];
                let value = chunk[1];
                let description = match register {
                    REG_SHUTDOWN => format!(
                        "Shutdown: {}",
                        if value == 1 { "Normal" } else { "Shutdown" }
                    ),
                    REG_DECODE_MODE => format!("Decode Mode: 0x{value:02X}"),
                    REG_SCAN_LIMIT => format!("Scan Limit: {} digits", value + 1),
                    REG_INTENSITY => format!("Intensity: {value} (0x{value:02X})"),
                    0x01..=0x08 => format!("Digit {}: 0x{:02X}", register - 1, value),
                    REG_NO_OP => "No Operation".to_string(),
                    _ => format!("Unknown Register 0x{register:02X}: 0x{value:02X}"),
                };
                println!("   └─ {description}");
            }
        }
        println!();
    }

    pub fn refresh_display(&mut self) {
        let mut data_to_send = vec![0u8; self.devices_in_string * 2];
        let mut refresh_required = true;
        let mut refresh_cycles = 0;

        println!("🔄 Starting display refresh...");

        while refresh_required {
            refresh_cycles += 1;
            refresh_required = false;

            println!("   Refresh cycle #{refresh_cycles}");

            for i in 0..self.devices_in_string {
                let device_data = self.drivers[i].get_data_to_output();
                data_to_send[i * 2] = device_data[0];
                data_to_send[i * 2 + 1] = device_data[1];

                refresh_required = refresh_required || self.drivers[i].data_to_refresh();
            }

            self.mock_spi(&data_to_send);
        }

        println!("✅ Display refresh completed in {refresh_cycles} cycles");
    }
}

fn main() {
    println!("MAX7219 Console Demo - No Hardware Required");
    println!("============================================");
    println!("This demo shows the SPI commands that would be sent to MAX7219 devices");
    println!("Based on the C# implementation from Downloads/MAX7219");
    println!();

    // Initialize mock driver with 8 devices, CS pin 24
    let mut display_driver = Max7219Driver::new(8, 24);
    println!("✅ Initialized mock MAX7219 driver with 8 devices, CS pin 24");

    loop {
        println!("\n🎮 MAX7219 Demo Menu:");
        println!("1. Run C# Example Pattern (shows exact SPI commands)");
        println!("2. Individual Device Configuration Demo");
        println!("3. Test Pattern Demo");
        println!("4. Brightness Sweep Demo");
        println!("5. Show Current Device States");
        println!("6. Reset All Devices");
        println!("7. Exit");
        print!("\nSelect option (1-7): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => run_csharp_example_demo(&mut display_driver),
            "2" => individual_config_demo(&mut display_driver),
            "3" => test_pattern_demo(&mut display_driver),
            "4" => brightness_sweep_demo(&mut display_driver),
            "5" => show_device_states(&display_driver),
            "6" => reset_all_devices(&mut display_driver),
            "7" => {
                println!("👋 Demo completed!");
                break;
            }
            _ => println!("❌ Invalid option. Please select 1-7."),
        }
    }
}

fn run_csharp_example_demo(driver: &mut Max7219Driver) {
    println!("\n🔄 C# Example Pattern Demo");
    println!("This shows the exact SPI commands from Form1.button1_Click");
    println!();

    // Configure all 8 devices exactly like the C# code
    for i in 0..8 {
        println!("📋 Configuring device {i}:");

        // Set digit data: each digit gets (1 << digit_position)
        for d in 0..8 {
            driver.drivers[i].set_digit(d as u8, 1 << d);
        }

        // Set intensity (i * 2)
        driver.drivers[i].set_intensity((i * 2) as u8);

        // Set digit decoder: BCD on digits 1 and 5
        driver.drivers[i].set_digit_decoder((1 << 1) | (1 << 5));

        // Set normal mode
        driver.drivers[i].set_shutdown(1);

        println!("   - Digits: [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80]");
        println!("   - Intensity: {}", i * 2);
        println!(
            "   - Decoder: 0x{:02X} (BCD on digits 1 and 5)",
            (1 << 1) | (1 << 5)
        );
        println!("   - Mode: Normal");
    }

    println!("\n📡 SPI Commands that would be sent:");
    driver.refresh_display();
}

fn individual_config_demo(driver: &mut Max7219Driver) {
    println!("\n⚙️  Individual Device Configuration Demo");
    println!("Configuring device 0 with custom settings...");

    // Configure device 0 with specific settings
    driver.drivers[0].set_intensity(10);
    driver.drivers[0].set_digit_decoder(0xFF); // BCD on all digits
    driver.drivers[0].set_digits_count(6); // Only use 6 digits

    // Set digits to display "123456"
    for digit in 0..6 {
        driver.drivers[0].set_digit(digit, digit + 1);
    }

    println!("📡 SPI Commands for individual configuration:");
    driver.refresh_display();
}

fn test_pattern_demo(driver: &mut Max7219Driver) {
    println!("\n🎯 Test Pattern Demo");

    // Pattern: Alternating segments
    println!("Setting alternating segment pattern...");
    for i in 0..driver.devices_in_string {
        for digit in 0..8 {
            let pattern = if (digit + i) % 2 == 0 { 0xAA } else { 0x55 };
            driver.drivers[i].set_digit(digit as u8, pattern);
        }
        driver.drivers[i].set_shutdown(1);
    }

    println!("📡 SPI Commands for test pattern:");
    driver.refresh_display();
}

fn brightness_sweep_demo(driver: &mut Max7219Driver) {
    println!("\n💡 Brightness Sweep Demo");
    println!("This would sweep brightness from 0 to 15...");

    // Set all devices to display "8" (all segments on)
    for i in 0..driver.devices_in_string {
        for digit in 0..8 {
            driver.drivers[i].set_digit(digit as u8, 0x7F); // All segments
        }
        driver.drivers[i].set_shutdown(1);
    }

    // Show what brightness commands would look like
    for brightness in [0, 5, 10, 15] {
        println!("\n📋 Setting brightness to {brightness}:");
        for i in 0..driver.devices_in_string {
            driver.drivers[i].set_intensity(brightness);
        }

        println!("📡 SPI Commands for brightness {brightness}:");
        driver.refresh_display();
    }
}

fn show_device_states(driver: &Max7219Driver) {
    println!("\n📊 Current Device States:");

    for (i, device) in driver.drivers.iter().enumerate() {
        println!("Device {i}:");
        println!(
            "   Shutdown: {} ({})",
            device.shutdown,
            if device.shutdown == 1 {
                "Normal"
            } else {
                "Shutdown"
            }
        );
        println!(
            "   Intensity: {} (0x{:02X})",
            device.intensity, device.intensity
        );
        println!("   Digit Decoder: 0x{:02X}", device.digit_decoder);
        println!("   Digits Count: {}", device.digits_count);
        print!("   Digit Data: [");
        for (j, &data) in device.digits_data.iter().enumerate() {
            if j > 0 {
                print!(", ");
            }
            print!("0x{data:02X}");
        }
        println!("]");

        print!("   Needs Refresh: [");
        for (j, &needs) in device.needs_refresh.iter().enumerate() {
            if j > 0 {
                print!(", ");
            }
            print!("{}", if needs { "Y" } else { "N" });
        }
        println!("]");
        println!();
    }
}

fn reset_all_devices(driver: &mut Max7219Driver) {
    println!("\n🔄 Resetting all devices to initial state...");

    for i in 0..driver.devices_in_string {
        driver.drivers[i] = Max7219Device::new();
    }

    println!("📡 SPI Commands for reset:");
    driver.refresh_display();

    println!("✅ All devices reset to initial state");
}
