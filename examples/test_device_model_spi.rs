//! Test Device Model SPI Capabilities
//!
//! This example verifies that device models correctly define SPI capabilities
//! and that pin validation works with the updated models.

use pokeys_lib::models::DeviceModel;
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Device Model SPI Capabilities");
    println!("=========================================");

    // Test all device models
    let model_files = [
        ("PoKeys56U", "models/PoKeys56U.yaml"),
        ("PoKeys56E", "models/PoKeys56E.yaml"),
        ("PoKeys57U", "models/PoKeys57U.yaml"),
        ("PoKeys57E", "models/PoKeys57E.yaml"),
    ];

    for (model_name, model_path) in &model_files {
        println!("\n📱 Testing {model_name} Model");
        println!("{}=", "=".repeat(model_name.len() + 15));

        // Load the device model
        let model = DeviceModel::from_file(model_path)?;
        println!("✅ Model loaded successfully: {}", model.name);

        // Test SPI pin capabilities
        test_spi_pin_capabilities(&model)?;

        // Test CS pin availability
        test_cs_pin_availability(&model)?;

        // Test model validation
        test_model_validation(&model)?;
    }

    println!("\n🎉 Device Model SPI Testing Complete!");
    println!("✅ All device models correctly define SPI capabilities");

    Ok(())
}

fn test_spi_pin_capabilities(model: &DeviceModel) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔌 Testing SPI Pin Capabilities");

    // Test pin 23 (MOSI)
    if let Some(pin_23) = model.pins.get(&23) {
        let has_spi_mosi = pin_23.capabilities.contains(&"SpiMosi".to_string());
        let has_digital_io = pin_23.capabilities.contains(&"DigitalInput".to_string())
            && pin_23.capabilities.contains(&"DigitalOutput".to_string());

        if has_spi_mosi {
            println!("  ✅ Pin 23: SpiMosi capability found");
        } else {
            println!("  ❌ Pin 23: Missing SpiMosi capability");
            return Err("Pin 23 should have SpiMosi capability".into());
        }

        if has_digital_io {
            println!("  ✅ Pin 23: Digital I/O capabilities preserved");
        } else {
            println!("  ❌ Pin 23: Missing digital I/O capabilities");
        }

        println!("  📋 Pin 23 capabilities: {:?}", pin_23.capabilities);
    } else {
        return Err("Pin 23 not found in model".into());
    }

    // Test pin 25 (CLK)
    if let Some(pin_25) = model.pins.get(&25) {
        let has_spi_clock = pin_25.capabilities.contains(&"SpiClock".to_string());
        let has_digital_io = pin_25.capabilities.contains(&"DigitalInput".to_string())
            && pin_25.capabilities.contains(&"DigitalOutput".to_string());

        if has_spi_clock {
            println!("  ✅ Pin 25: SpiClock capability found");
        } else {
            println!("  ❌ Pin 25: Missing SpiClock capability");
            return Err("Pin 25 should have SpiClock capability".into());
        }

        if has_digital_io {
            println!("  ✅ Pin 25: Digital I/O capabilities preserved");
        } else {
            println!("  ❌ Pin 25: Missing digital I/O capabilities");
        }

        println!("  📋 Pin 25 capabilities: {:?}", pin_25.capabilities);
    } else {
        return Err("Pin 25 not found in model".into());
    }

    Ok(())
}

fn test_cs_pin_availability(model: &DeviceModel) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📺 Testing CS Pin Availability");

    let mut cs_capable_pins = Vec::new();
    let mut total_pins = 0;

    for (pin_num, pin) in &model.pins {
        total_pins += 1;
        if pin.capabilities.contains(&"SpiChipSelect".to_string()) {
            cs_capable_pins.push(*pin_num);
        }
    }

    cs_capable_pins.sort();

    println!("  📊 Total pins in model: {total_pins}");
    println!("  📌 CS-capable pins: {} pins", cs_capable_pins.len());
    println!("  📋 CS pins: {cs_capable_pins:?}");

    // Verify recommended CS pins are available
    let recommended_cs_pins = [24, 26, 27, 28, 29, 30];
    let mut available_recommended = Vec::new();

    for &pin in &recommended_cs_pins {
        if cs_capable_pins.contains(&pin) {
            available_recommended.push(pin);
        }
    }

    println!("  ✅ Recommended CS pins available: {available_recommended:?}");

    // Verify SPI pins are NOT CS-capable (they shouldn't be used as CS)
    if cs_capable_pins.contains(&23) {
        println!("  ⚠️  Pin 23 (MOSI) is CS-capable - this is allowed but not recommended");
    }
    if cs_capable_pins.contains(&25) {
        println!("  ⚠️  Pin 25 (CLK) is CS-capable - this is allowed but not recommended");
    }

    if cs_capable_pins.len() < 5 {
        return Err("Model should have at least 5 CS-capable pins for multi-display setups".into());
    }

    Ok(())
}

fn test_model_validation(model: &DeviceModel) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️ Testing Model Validation");

    // Test that the model validates successfully
    match model.validate() {
        Ok(()) => println!("  ✅ Model validation passed"),
        Err(e) => {
            println!("  ❌ Model validation failed: {e}");
            return Err(format!("Model validation failed: {e}").into());
        }
    }

    // Test that all pins have at least one capability
    let mut pins_without_capabilities = Vec::new();
    for (pin_num, pin) in &model.pins {
        if pin.capabilities.is_empty() {
            pins_without_capabilities.push(*pin_num);
        }
    }

    if pins_without_capabilities.is_empty() {
        println!("  ✅ All pins have capabilities defined");
    } else {
        println!("  ❌ Pins without capabilities: {pins_without_capabilities:?}");
        return Err("All pins must have at least one capability".into());
    }

    // Test that SPI capabilities are properly defined
    let spi_capabilities = ["SpiMosi", "SpiClock", "SpiChipSelect", "SpiMiso"];
    let mut found_spi_capabilities = HashSet::new();

    for pin in model.pins.values() {
        for capability in &pin.capabilities {
            if spi_capabilities.contains(&capability.as_str()) {
                found_spi_capabilities.insert(capability.clone());
            }
        }
    }

    println!("  📋 SPI capabilities found: {found_spi_capabilities:?}");

    // Verify essential SPI capabilities are present
    if found_spi_capabilities.contains("SpiMosi") {
        println!("  ✅ SpiMosi capability found");
    } else {
        return Err("SpiMosi capability not found in model".into());
    }

    if found_spi_capabilities.contains("SpiClock") {
        println!("  ✅ SpiClock capability found");
    } else {
        return Err("SpiClock capability not found in model".into());
    }

    if found_spi_capabilities.contains("SpiChipSelect") {
        println!("  ✅ SpiChipSelect capability found");
    } else {
        return Err("SpiChipSelect capability not found in model".into());
    }

    Ok(())
}
