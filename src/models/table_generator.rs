use super::{DeviceModel, PinModel};
use std::collections::HashMap;

/// Generate a Markdown table of pin capabilities for a device model
///
/// # Arguments
///
/// * `model` - The device model to generate a table for
///
/// # Returns
///
/// * `String` - Markdown table of pin capabilities
pub fn generate_pin_capability_table(model: &DeviceModel) -> String {
    let mut table = String::new();

    // Add table header
    table.push_str("# Pin Capabilities for ");
    table.push_str(&model.name);
    table.push_str("\n\n");

    // Add table header row
    table.push_str("| Pin | Digital Input | Digital Output | Analog Input | Special Functions |\n");
    table.push_str("|-----|--------------|----------------|--------------|-------------------|\n");

    // Sort pins by number
    let mut pins: Vec<(&u8, &PinModel)> = model.pins.iter().collect();
    pins.sort_by_key(|&(pin, _)| pin);

    // Add rows for each pin
    for (pin, pin_model) in pins {
        let digital_input = if pin_model.capabilities.contains(&"DigitalInput".to_string()) {
            "✓"
        } else {
            " "
        };

        let digital_output = if pin_model.capabilities.contains(&"DigitalOutput".to_string()) {
            "✓"
        } else {
            " "
        };

        let analog_input = if pin_model.capabilities.contains(&"AnalogInput".to_string()) {
            "✓"
        } else {
            " "
        };

        // Collect special functions
        let special_functions: Vec<String> = pin_model.capabilities.iter()
            .filter(|&cap| {
                !cap.contains("DigitalInput") &&
                !cap.contains("DigitalOutput") &&
                !cap.contains("AnalogInput")
            })
            .cloned()
            .collect();

        let special = if special_functions.is_empty() {
            " ".to_string()
        } else {
            special_functions.join(", ")
        };

        // Add row
        table.push_str(&format!("| {} | {} | {} | {} | {} |\n", pin, digital_input, digital_output, analog_input, special));
    }

    // Add encoder pairs section if any
    let mut encoder_pairs = HashMap::new();

    for (pin, pin_model) in &model.pins {
        for capability in &pin_model.capabilities {
            if capability.starts_with("Encoder_") && capability.len() >= 10 {
                let encoder_id = &capability[8..capability.len() - 1]; // Extract "1" from "Encoder_1A"
                let role = &capability[capability.len() - 1..]; // Extract "A" from "Encoder_1A"

                encoder_pairs
                    .entry(encoder_id.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(role.to_string(), *pin);
            }
        }
    }

    if !encoder_pairs.is_empty() {
        table.push_str("\n## Encoder Pairs\n\n");
        table.push_str("| Encoder | Pin A | Pin B |\n");
        table.push_str("|---------|-------|-------|\n");

        // Sort encoder pairs by ID
        let mut encoder_ids: Vec<&String> = encoder_pairs.keys().collect();
        encoder_ids.sort();

        for encoder_id in encoder_ids {
            let pins = encoder_pairs.get(encoder_id).unwrap();
            let pin_a = pins.get("A").unwrap_or(&0);
            let pin_b = pins.get("B").unwrap_or(&0);

            table.push_str(&format!("| {} | {} | {} |\n", encoder_id, pin_a, pin_b));
        }
    }

    // Add matrix keyboard section if any
    let mut matrix_rows = Vec::new();
    let mut matrix_cols = Vec::new();

    for (pin, pin_model) in &model.pins {
        for capability in &pin_model.capabilities {
            if capability.starts_with("MatrixKeyboard_Row") {
                matrix_rows.push((*pin, capability.clone()));
            } else if capability.starts_with("MatrixKeyboard_Col") {
                matrix_cols.push((*pin, capability.clone()));
            }
        }
    }

    if !matrix_rows.is_empty() && !matrix_cols.is_empty() {
        table.push_str("\n## Matrix Keyboard\n\n");

        // Sort rows and columns
        matrix_rows.sort_by(|a, b| a.1.cmp(&b.1));
        matrix_cols.sort_by(|a, b| a.1.cmp(&b.1));

        table.push_str("### Rows\n\n");
        table.push_str("| Pin | Row |\n");
        table.push_str("|-----|-----|\n");

        for (pin, capability) in &matrix_rows {
            let row = &capability[17..]; // Extract "1" from "MatrixKeyboard_Row1"
            table.push_str(&format!("| {} | {} |\n", pin, row));
        }

        table.push_str("\n### Columns\n\n");
        table.push_str("| Pin | Column |\n");
        table.push_str("|-----|--------|\n");

        for (pin, capability) in &matrix_cols {
            let col = &capability[17..]; // Extract "1" from "MatrixKeyboard_Col1"
            table.push_str(&format!("| {} | {} |\n", pin, col));
        }
    }

    table
}
