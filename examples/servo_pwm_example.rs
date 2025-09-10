use pokeys_lib::*;
use std::io::{self, Write};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

fn main() -> Result<()> {
    println!("PoKeys Servo Calibration Tool - Pin 22");
    println!("=====================================");
    
    match connect_to_first_available_device() {
        Ok(mut device) => run_servo_calibration(&mut device),
        Err(_) => {
            println!("No PoKeys device found. Connect hardware and try again.");
            Ok(())
        }
    }
}

#[cfg(unix)]
fn getch() -> Result<char> {
    use std::os::unix::io::AsRawFd;
    let stdin = std::io::stdin();
    let fd = stdin.as_raw_fd();
    
    // Set terminal to raw mode
    let mut termios = unsafe { std::mem::zeroed() };
    unsafe { libc::tcgetattr(fd, &mut termios) };
    let original = termios;
    
    termios.c_lflag &= !(libc::ICANON | libc::ECHO);
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) };
    
    // Read single character
    let mut buf = [0u8; 1];
    let result = match std::io::Read::read(&mut std::io::stdin(), &mut buf) {
        Ok(_) => Ok(buf[0] as char),
        Err(e) => Err(PoKeysError::InternalError(format!("Input error: {}", e))),
    };
    
    // Restore terminal
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original) };
    
    result
}

#[cfg(windows)]
fn getch() -> Result<char> {
    use std::os::windows::io::AsRawHandle;
    // Windows implementation would go here
    Err(PoKeysError::InternalError("Single char input not implemented on Windows".to_string()))
}

fn run_servo_calibration(device: &mut PoKeysDevice) -> Result<()> {
    let servo_pin: u8 = 22;
    
    println!("✓ Connected to PoKeys device");
    println!("Setting up PWM on pin {}", servo_pin);
    
    device.set_pwm_period(500000)?;
    device.enable_pwm_for_pin(servo_pin, true)?;
    
    let mut current_duty = 37500u32;
    device.set_pwm_duty_cycle_for_pin(servo_pin, current_duty)?;
    
    println!("\nServo Calibration (single key press):");
    println!("+ - : increase/decrease | s l : small/large steps");
    println!("0 9 1 : presets (0°/90°/180°) | q : quit");
    println!("Duty: {} | Step: 1000\n", current_duty);
    
    let mut step_size = 1000u32;
    
    loop {
        print!("Press key: ");
        io::stdout().flush().unwrap();
        
        let ch = match getch() {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        match ch {
            '+' | '=' => {
                current_duty = (current_duty + step_size).min(60000);
                device.set_pwm_duty_cycle_for_pin(servo_pin, current_duty)?;
                println!("+ → {}", current_duty);
            }
            '-' | '_' => {
                current_duty = current_duty.saturating_sub(step_size);
                device.set_pwm_duty_cycle_for_pin(servo_pin, current_duty)?;
                println!("- → {}", current_duty);
            }
            's' => {
                step_size = 250;
                println!("s → step: {}", step_size);
            }
            'l' => {
                step_size = 2500;
                println!("l → step: {}", step_size);
            }
            '0' => {
                current_duty = 60000;
                device.set_pwm_duty_cycle_for_pin(servo_pin, current_duty)?;
                println!("0 → {} (0°)", current_duty);
            }
            '9' => {
                current_duty = 36000;
                device.set_pwm_duty_cycle_for_pin(servo_pin, current_duty)?;
                println!("9 → {} (90°)", current_duty);
            }
            '1' => {
                current_duty = 12000;
                device.set_pwm_duty_cycle_for_pin(servo_pin, current_duty)?;
                println!("1 → {} (180°)", current_duty);
            }
            'q' | '\x1b' => break, // q or ESC
            _ => println!("{} → unknown", ch),
        }
    }
    
    device.enable_pwm_for_pin(servo_pin, false)?;
    println!("\n✓ PWM disabled");
    
    Ok(())
}

fn connect_to_first_available_device() -> Result<PoKeysDevice> {
    if enumerate_usb_devices()? > 0 {
        return connect_to_device(0);
    }
    
    let network_devices = enumerate_network_devices(1000)?;
    if !network_devices.is_empty() {
        return connect_to_network_device(&network_devices[0]);
    }
    
    Err(PoKeysError::DeviceNotFound)
}
