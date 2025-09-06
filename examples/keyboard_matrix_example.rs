//! Keyboard Matrix Example
//!
//! This example demonstrates how to configure and use the matrix keyboard functionality
//! with PoKeys devices. It shows:
//! - Configuring a matrix keyboard with specific pins
//! - Reading keyboard state
//! - Monitoring key presses and releases
//!
//! Run with: cargo run --example keyboard_matrix_example

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_range_loop)]

use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🎹 PoKeys Matrix Keyboard Example");
    println!("=================================");

    // Try to connect to a device
    let mut device = match connect_to_first_available_device() {
        Ok(device) => device,
        Err(e) => {
            println!("❌ No PoKeys device found: {e}");
            println!("💡 Please connect a PoKeys device to run this example");
            return Ok(());
        }
    };

    println!(
        "✅ Connected to device (Serial: {})",
        device.device_data.serial_number
    );

    // Configure a 4x4 matrix keyboard
    let width = 4;
    let height = 4;

    // Define pin assignments for the matrix
    // Columns: pins 21, 22, 23, 24 (first 4 pins)
    let column_pins = [5, 6, 7, 8];
    // Rows: pins 13, 14, 15, 16 (first 4 pins)
    let row_pins = [1, 2, 3, 4];

    println!("\n🔧 Configuring {width}x{height} matrix keyboard...");
    println!("   Column pins: {:?}", &column_pins[..width as usize]);
    println!("   Row pins: {:?}", &row_pins[..height as usize]);

    // Configure the matrix keyboard
    device.configure_matrix_keyboard(
        width,
        height,
        &column_pins[..width as usize],
        &row_pins[..height as usize],
    )?;

    println!("✅ Matrix keyboard configured successfully!");

    // Display the keyboard layout
    display_keyboard_layout(width, height);

    // Monitor keyboard for key presses
    println!("\n🔍 Monitoring keyboard (press Ctrl+C to exit)...");
    println!("Press keys on the matrix keyboard to see their state changes.");

    let mut previous_states = vec![vec![false; width as usize]; height as usize];
    let mut key_press_count = 0;

    loop {
        // Read the current keyboard state
        device.read_matrix_keyboard()?;

        // Check for state changes
        let mut state_changed = false;
        for (row, row_states) in previous_states.iter_mut().enumerate().take(height as usize) {
            for col in 0..width as usize {
                let current_state = device.matrix_keyboard.get_key_state(row, col);
                let previous_state = row_states[col];

                if current_state != previous_state {
                    state_changed = true;
                    if current_state {
                        key_press_count += 1;
                        println!("🔴 Key PRESSED  at ({row}, {col}) - Key #{key_press_count}");
                    } else {
                        println!("🔵 Key RELEASED at ({row}, {col})");
                    }
                    row_states[col] = current_state;
                }
            }
        }

        // Display current keyboard state if there were changes
        if state_changed {
            display_current_state(&device, width, height);
        }

        // Small delay to avoid overwhelming the output
        thread::sleep(Duration::from_millis(50));
    }
}

fn connect_to_first_available_device() -> Result<PoKeysDevice> {
    // Try network devices first
    println!("🔍 Searching for network devices...");
    match enumerate_network_devices(4000) {
        Ok(devices) if !devices.is_empty() => {
            println!("✅ Found {} network device(s)", devices.len());
            return connect_to_network_device(&devices[0]);
        }
        Ok(_) => println!("ℹ️  No network devices found"),
        Err(e) => println!("⚠️  Network enumeration failed: {e}"),
    }

    // Try USB devices
    println!("🔍 Searching for USB devices...");
    match enumerate_usb_devices() {
        Ok(count) if count > 0 => {
            println!("✅ Found {count} USB device(s)");
            connect_to_device(0)
        }
        Ok(_) => Err(PoKeysError::DeviceNotFound),
        Err(e) => Err(e),
    }
}

fn display_keyboard_layout(width: u8, height: u8) {
    println!("\n📋 Keyboard Layout ({width}x{height}):");
    println!("   ┌─────┬─────┬─────┬─────┐");

    for row in 0..height {
        print!("   │");
        for col in 0..width {
            print!(" {row:>2},{col} │");
        }
        println!();

        if row < height - 1 {
            println!("   ├─────┼─────┼─────┼─────┤");
        }
    }

    println!("   └─────┴─────┴─────┴─────┘");
    println!("   Format: (row,col) - e.g., (0,0) is top-left");
}

fn display_current_state(device: &PoKeysDevice, width: u8, height: u8) {
    println!("\n📊 Current Keyboard State:");
    println!("   ┌─────┬─────┬─────┬─────┐");

    for row in 0..height {
        print!("   │");
        for col in 0..width {
            let pressed = device
                .matrix_keyboard
                .get_key_state(row as usize, col as usize);
            let symbol = if pressed { " ███ " } else { "     " };
            print!("{symbol}│");
        }
        println!();

        if row < height - 1 {
            println!("   ├─────┼─────┼─────┼─────┤");
        }
    }

    println!("   └─────┴─────┴─────┴─────┘");
    println!("   ███ = pressed, blank = released");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_functions() {
        // Test that our helper functions don't panic
        display_keyboard_layout(4, 4);

        // Create a mock device for testing display_current_state
        // This would require more setup in a real test, but demonstrates the concept
    }
}
