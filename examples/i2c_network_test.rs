//! Network I2C Test Example
//!
//! This example demonstrates I2C operations over network connection
//! with configurable device serial number.
//!
//! Usage: 
//!   cargo run --example i2c_network_test -- --serial 32218
//!   cargo run --example i2c_network_test -- --serial 12345 --address 0x68
//!   cargo run --example i2c_network_test -- --help

use pokeys_lib::*;
use std::env;

fn main() -> Result<()> {
    println!("🔧 Network I2C Test Example");
    println!("===========================");

    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let (device_serial, test_address) = parse_args(&args)?;
    
    println!("🌐 Connecting to device {} over network...", device_serial);
    
    let mut device = match connect_to_device_with_serial(device_serial, true, 5000) {
        Ok(device) => {
            println!("✅ Successfully connected to device {}", device_serial);
            device
        }
        Err(e) => {
            println!("❌ Failed to connect to device {}: {}", device_serial, e);
            println!("💡 Troubleshooting:");
            println!("   - Ensure device {} is powered on", device_serial);
            println!("   - Check network connectivity");
            println!("   - Verify the serial number is correct");
            println!("   - Try: ping <device_ip> to test network connectivity");
            return Ok(());
        }
    };
    
    // Get and display device information
    device.get_device_data()?;
    let device_name = String::from_utf8_lossy(&device.device_data.device_name);
    println!("🔗 Connected to: {} (Serial: {})", 
             device_name.trim_end_matches('\0'), 
             device.device_data.serial_number);
    
    println!("📊 Device Info:");
    println!("   - Firmware Version: {}.{}", 
             device.device_data.firmware_version_major,
             device.device_data.firmware_version_minor);
    println!("   - Hardware Type: {}", device.device_data.hw_type);
    println!("   - Device Type: {}", device.device_data.device_type);

    // Initialize I2C
    println!("\n🚀 Initializing I2C...");
    match device.i2c_init() {
        Ok(()) => println!("✅ I2C initialized successfully"),
        Err(e) => {
            println!("❌ I2C initialization failed: {}", e);
            return Ok(());
        }
    }

    // Note: i2c_configure is not yet implemented in the current version
    // The i2c_init() function initializes with default settings (100kHz)
    println!("⚙️  I2C configured with default settings (100kHz standard speed)");

    // Perform I2C operations
    println!("\n🧪 Testing I2C operations with device at 0x{:02X}", test_address);
    identify_and_test_device(&mut device, test_address)?;

    println!("\n🎉 Network I2C Test Complete!");
    Ok(())
}

fn parse_args(args: &[String]) -> Result<(u32, u8)> {
    let mut device_serial = 32218u32; // Default serial number
    let mut test_address = 0x50u8;    // Default I2C address (EEPROM)
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--serial" | "-s" => {
                if i + 1 < args.len() {
                    device_serial = args[i + 1].parse()
                        .map_err(|_| PoKeysError::Parameter("Invalid serial number".to_string()))?;
                    i += 2;
                } else {
                    return Err(PoKeysError::Parameter("--serial requires a value".to_string()));
                }
            }
            "--address" | "-a" => {
                if i + 1 < args.len() {
                    let addr_str = &args[i + 1];
                    test_address = if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
                        u8::from_str_radix(&addr_str[2..], 16)
                    } else {
                        addr_str.parse()
                    }.map_err(|_| PoKeysError::Parameter("Invalid I2C address".to_string()))?;
                    i += 2;
                } else {
                    return Err(PoKeysError::Parameter("--address requires a value".to_string()));
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                println!("⚠️  Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }
    
    Ok((device_serial, test_address))
}

fn print_help() {
    println!("Network I2C Test Example");
    println!();
    println!("USAGE:");
    println!("    cargo run --example i2c_network_test -- [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -s, --serial <SERIAL>     Device serial number [default: 32218]");
    println!("    -a, --address <ADDRESS>   I2C device address to test [default: 0x50]");
    println!("    -h, --help               Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run --example i2c_network_test -- --serial 32218");
    println!("    cargo run --example i2c_network_test -- --serial 12345 --address 0x68");
    println!("    cargo run --example i2c_network_test -- -s 32218 -a 80");
    println!();
    println!("COMMON I2C ADDRESSES:");
    println!("    0x50-0x57    EEPROM (24LC series)");
    println!("    0x68         Real-Time Clock (DS1307/DS3231)");
    println!("    0x48-0x4F    Temperature Sensor (LM75/DS18B20)");
    println!("    0x3C, 0x3D   OLED Display (SSD1306)");
    println!("    0x20-0x27    I/O Expander (PCF8574)");
}

fn identify_and_test_device(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    let device_type = identify_device_type(address);
    if !device_type.is_empty() {
        println!("🔍 Device 0x{:02X} appears to be: {}", address, device_type);
    }

    match address {
        0x50..=0x57 => test_eeprom(device, address),
        0x68 => test_rtc(device, address),
        0x48..=0x4F => test_temperature_sensor(device, address),
        _ => test_generic_device(device, address),
    }
}

fn test_eeprom(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("💾 Testing EEPROM operations...");
    
    let memory_addr = 0x0010u16; // Use address 0x0010 to avoid overwriting important data
    let test_data = b"PoKeys I2C Test";
    
    println!("📝 Writing '{}' to EEPROM address 0x{:04X}", 
             String::from_utf8_lossy(test_data), memory_addr);
    
    // Write: memory address (2 bytes) + data
    let mut write_data = Vec::new();
    write_data.push((memory_addr >> 8) as u8);  // High byte
    write_data.push((memory_addr & 0xFF) as u8); // Low byte
    write_data.extend_from_slice(test_data);
    
    match device.i2c_write(address, &write_data) {
        Ok(I2cStatus::Ok) => {
            println!("✅ EEPROM write successful");
            
            // Wait for write cycle
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            // Set read address
            let addr_bytes = vec![(memory_addr >> 8) as u8, (memory_addr & 0xFF) as u8];
            match device.i2c_write(address, &addr_bytes) {
                Ok(I2cStatus::Ok) => {
                    // Read back
                    match device.i2c_read(address, test_data.len() as u8) {
                        Ok((I2cStatus::Ok, data)) => {
                            let read_str = String::from_utf8_lossy(&data);
                            println!("📖 EEPROM read: '{}'", read_str);
                            
                            if data == test_data {
                                println!("✅ EEPROM test PASSED!");
                            } else {
                                println!("⚠️  Data mismatch - this could be normal if:");
                                println!("   - EEPROM is write-protected");
                                println!("   - Different data was already stored");
                                println!("   - EEPROM requires different timing");
                            }
                        }
                        Ok((status, data)) => println!("⚠️  EEPROM read status: {:?}, data: {:02X?}", status, data),
                        Err(e) => println!("❌ EEPROM read failed: {}", e),
                    }
                }
                Ok(status) => println!("⚠️  EEPROM address set status: {:?}", status),
                Err(e) => println!("❌ EEPROM address set failed: {}", e),
            }
        }
        Ok(status) => println!("⚠️  EEPROM write status: {:?}", status),
        Err(e) => println!("❌ EEPROM write failed: {}", e),
    }
    
    Ok(())
}

fn test_rtc(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("🕐 Testing Real-Time Clock operations...");
    
    // Try to read current time (registers 0x00-0x06)
    let time_addr = 0x00;
    match device.i2c_write(address, &[time_addr]) {
        Ok(I2cStatus::Ok) => {
            match device.i2c_read(address, 7) {
                Ok((I2cStatus::Ok, data)) => {
                    if data.len() >= 7 {
                        // Decode BCD time format
                        let seconds = bcd_to_decimal(data[0] & 0x7F);
                        let minutes = bcd_to_decimal(data[1]);
                        let hours = bcd_to_decimal(data[2] & 0x3F);
                        let day = bcd_to_decimal(data[3]);
                        let date = bcd_to_decimal(data[4]);
                        let month = bcd_to_decimal(data[5]);
                        let year = bcd_to_decimal(data[6]);
                        
                        println!("📅 RTC Time: 20{:02}-{:02}-{:02} {:02}:{:02}:{:02} (Day {})",
                                year, month, date, hours, minutes, seconds, day);
                        
                        if data[0] & 0x80 != 0 {
                            println!("⚠️  RTC clock is stopped (CH bit set)");
                        } else {
                            println!("✅ RTC clock is running");
                        }
                    }
                }
                Ok((status, data)) => println!("⚠️  RTC read status: {:?}, data: {:02X?}", status, data),
                Err(e) => println!("❌ RTC read failed: {}", e),
            }
        }
        Ok(status) => println!("⚠️  RTC address set status: {:?}", status),
        Err(e) => println!("❌ RTC address set failed: {}", e),
    }
    
    Ok(())
}

fn test_temperature_sensor(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("🌡️  Testing temperature sensor...");
    
    // Try to read temperature (typically register 0x00)
    let temp_reg = 0x00;
    match device.i2c_write(address, &[temp_reg]) {
        Ok(I2cStatus::Ok) => {
            match device.i2c_read(address, 2) {
                Ok((I2cStatus::Ok, data)) => {
                    if data.len() >= 2 {
                        // Assume LM75-style 16-bit temperature
                        let temp_raw = ((data[0] as u16) << 8) | (data[1] as u16);
                        let temperature = (temp_raw as i16) as f32 / 256.0;
                        println!("🌡️  Temperature: {:.2}°C", temperature);
                        println!("✅ Temperature sensor read successful");
                    }
                }
                Ok((status, data)) => println!("⚠️  Temperature sensor status: {:?}, data: {:02X?}", status, data),
                Err(e) => println!("❌ Temperature sensor read failed: {}", e),
            }
        }
        Ok(status) => println!("⚠️  Temperature sensor address set status: {:?}", status),
        Err(e) => println!("❌ Temperature sensor address set failed: {}", e),
    }
    
    Ok(())
}

fn test_generic_device(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("🔧 Testing generic I2C device...");
    
    // Try basic write/read operations
    let test_data = vec![0x01, 0x02, 0x03];
    println!("📤 Writing test data: {:02X?}", test_data);
    
    match device.i2c_write(address, &test_data) {
        Ok(I2cStatus::Ok) => {
            println!("✅ Write successful");
            
            // Try to read back
            match device.i2c_read(address, 3) {
                Ok((I2cStatus::Ok, data)) => {
                    println!("📥 Read successful: {:02X?}", data);
                }
                Ok((status, data)) => println!("⚠️  Read status: {:?}, data: {:02X?}", status, data),
                Err(e) => println!("❌ Read failed: {}", e),
            }
        }
        Ok(status) => println!("⚠️  Write status: {:?}", status),
        Err(e) => println!("❌ Write failed: {}", e),
    }
    
    Ok(())
}

fn identify_device_type(address: u8) -> String {
    match address {
        0x50..=0x57 => "EEPROM (24LC series)".to_string(),
        0x68 => "Real-Time Clock (DS1307/DS3231)".to_string(),
        0x48..=0x4F => "Temperature Sensor (LM75/DS18B20)".to_string(),
        0x3C | 0x3D => "OLED Display (SSD1306)".to_string(),
        0x20..=0x27 => "I/O Expander (PCF8574)".to_string(),
        0x38..=0x3F => "I/O Expander (PCF8574A)".to_string(),
        0x40..=0x47 => "PWM Driver (PCA9685)".to_string(),
        0x60..=0x67 => "PWM Driver (PCA9685)".to_string(),
        0x1E => "Magnetometer (HMC5883L)".to_string(),
        0x77 => "Pressure Sensor (BMP180/BMP280)".to_string(),
        _ => String::new(),
    }
}

fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}
