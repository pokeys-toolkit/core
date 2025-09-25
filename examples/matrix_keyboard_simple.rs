//! Simple Matrix Keyboard Example
//!
//! This is a minimal example showing matrix keyboard configuration and monitoring.
//! Run with: cargo run --example matrix_keyboard_simple

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_range_loop)]

use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Simple Matrix Keyboard Example");
    println!("==============================");

    // Connect to first available device
    let mut device = match connect_to_first_available_device() {
        Ok(device) => device,
        Err(e) => {
            println!("No PoKeys device found: {e}");
            println!("Please connect a PoKeys device and try again");
            return Ok(());
        }
    };

    println!("Connected to PoKeys device");

    // Configure 4x4 matrix keyboard
    let column_pins = [5, 6, 7, 8]; // Column pins 5-8
    let row_pins = [1, 2, 3, 4]; // Row pins 1-4

    device.configure_matrix_keyboard(4, 4, &column_pins, &row_pins)?;
    println!("Matrix keyboard configured successfully!");

    // Monitor for key presses
    let mut previous_states = vec![vec![false; 4]; 4];
    println!("Monitoring keyboard - press keys to see output...");

    loop {
        // Read current keyboard state
        device.read_matrix_keyboard()?;

        // Check each key position for changes
        for (row, row_states) in previous_states.iter_mut().enumerate().take(4) {
            for col in 0..4 {
                let current_state = device.matrix_keyboard.get_key_state(row, col);

                // Detect state changes
                if current_state != row_states[col] {
                    if current_state {
                        println!("Key PRESSED at position ({row}, {col})");
                    } else {
                        println!("Key RELEASED at position ({row}, {col})");
                    }
                    row_states[col] = current_state;
                }
            }
        }

        // Small delay to avoid overwhelming output
        thread::sleep(Duration::from_millis(50));
    }
}

fn connect_to_first_available_device() -> Result<PoKeysDevice> {
    // Try network devices first
    match enumerate_network_devices(2000) {
        Ok(devices) if !devices.is_empty() => {
            return connect_to_network_device(&devices[0]);
        }
        _ => {}
    }

    // Try USB devices
    match enumerate_usb_devices() {
        Ok(count) if count > 0 => connect_to_device(0),
        _ => Err(PoKeysError::DeviceNotFound),
    }
}
