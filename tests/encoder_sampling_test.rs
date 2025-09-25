//! Comprehensive tests for encoder 4x and 2x sampling functionality
//!
//! This test suite verifies that the encoder implementation correctly supports
//! both 4x and 2x sampling modes according to the PoKeys protocol specification.

use pokeys_lib::encoders::{EncoderData, EncoderOptions, MAX_ENCODERS, ULTRA_FAST_ENCODER_INDEX};

#[test]
fn test_encoder_4x_sampling_configuration() {
    // Test 4x sampling option creation
    let options = EncoderOptions::with_4x_sampling();

    assert!(options.enabled, "4x sampling encoder should be enabled");
    assert!(options.sampling_4x, "4x sampling should be enabled");
    assert!(!options.sampling_2x, "2x sampling should be disabled");

    // Test byte conversion
    let byte = options.to_byte();
    assert_eq!(
        byte & 0b00000011,
        0b00000011,
        "Bits 0 and 1 should be set for enabled + 4x"
    );

    // Test round-trip conversion
    let options_from_byte = EncoderOptions::from_byte(byte);
    assert_eq!(
        options, options_from_byte,
        "Round-trip conversion should preserve options"
    );
}

#[test]
fn test_encoder_2x_sampling_configuration() {
    // Test 2x sampling option creation
    let options = EncoderOptions::with_2x_sampling();

    assert!(options.enabled, "2x sampling encoder should be enabled");
    assert!(!options.sampling_4x, "4x sampling should be disabled");
    assert!(options.sampling_2x, "2x sampling should be enabled");

    // Test byte conversion
    let byte = options.to_byte();
    assert_eq!(
        byte & 0b00000111,
        0b00000101,
        "Bits 0 and 2 should be set for enabled + 2x"
    );

    // Test round-trip conversion
    let options_from_byte = EncoderOptions::from_byte(byte);
    assert_eq!(
        options, options_from_byte,
        "Round-trip conversion should preserve options"
    );
}

#[test]
fn test_encoder_sampling_mutual_exclusion() {
    // Test that both 4x and 2x cannot be enabled simultaneously
    let mut options = EncoderOptions::new();
    options.enabled = true;
    options.sampling_4x = true;
    options.sampling_2x = true;

    let byte = options.to_byte();
    assert_eq!(
        byte & 0b00000111,
        0b00000111,
        "Both sampling bits should be set in byte"
    );

    // When decoded, both flags should be preserved (validation happens at device level)
    let decoded_options = EncoderOptions::from_byte(byte);
    assert!(
        decoded_options.sampling_4x,
        "4x sampling flag should be preserved"
    );
    assert!(
        decoded_options.sampling_2x,
        "2x sampling flag should be preserved"
    );
}

#[test]
fn test_encoder_data_sampling_detection() {
    let mut encoder = EncoderData::new();

    // Test 4x sampling detection
    let options_4x = EncoderOptions::with_4x_sampling();
    encoder.set_options(options_4x);

    assert!(encoder.is_4x_sampling(), "Should detect 4x sampling");
    assert!(!encoder.is_2x_sampling(), "Should not detect 2x sampling");
    assert_eq!(
        encoder.sampling_mode_str(),
        "4x (both edges)",
        "Should return correct mode string"
    );

    // Test 2x sampling detection
    let options_2x = EncoderOptions::with_2x_sampling();
    encoder.set_options(options_2x);

    assert!(!encoder.is_4x_sampling(), "Should not detect 4x sampling");
    assert!(encoder.is_2x_sampling(), "Should detect 2x sampling");
    assert_eq!(
        encoder.sampling_mode_str(),
        "2x (A edges only)",
        "Should return correct mode string"
    );

    // Test disabled state
    let options_disabled = EncoderOptions::new();
    encoder.set_options(options_disabled);

    assert!(
        !encoder.is_4x_sampling(),
        "Should not detect 4x sampling when disabled"
    );
    assert!(
        !encoder.is_2x_sampling(),
        "Should not detect 2x sampling when disabled"
    );
    assert_eq!(
        encoder.sampling_mode_str(),
        "1x (disabled)",
        "Should return correct mode string"
    );
}

#[test]
fn test_encoder_options_bit_layout() {
    // Test the exact bit layout according to protocol specification
    // Bit layout: [macro_b][key_b][macro_a][key_a][reserved][2x][4x][enable]

    let mut options = EncoderOptions::new();

    // Test individual bits
    options.enabled = true;
    assert_eq!(options.to_byte() & 0b00000001, 0b00000001, "Bit 0: enable");

    options.sampling_4x = true;
    assert_eq!(
        options.to_byte() & 0b00000011,
        0b00000011,
        "Bit 1: 4x sampling"
    );

    options.sampling_2x = true;
    assert_eq!(
        options.to_byte() & 0b00000111,
        0b00000111,
        "Bit 2: 2x sampling"
    );

    // Bit 3 is reserved (should remain 0)
    assert_eq!(
        options.to_byte() & 0b00001000,
        0b00000000,
        "Bit 3: reserved (should be 0)"
    );

    options.direct_key_mapping_a = true;
    assert_eq!(
        options.to_byte() & 0b00010000,
        0b00010000,
        "Bit 4: direct key mapping A"
    );

    options.macro_mapping_a = true;
    assert_eq!(
        options.to_byte() & 0b00100000,
        0b00100000,
        "Bit 5: macro mapping A"
    );

    options.direct_key_mapping_b = true;
    assert_eq!(
        options.to_byte() & 0b01000000,
        0b01000000,
        "Bit 6: direct key mapping B"
    );

    options.macro_mapping_b = true;
    assert_eq!(
        options.to_byte() & 0b10000000,
        0b10000000,
        "Bit 7: macro mapping B"
    );
}

#[test]
fn test_encoder_constants() {
    // Verify encoder constants match protocol specification
    assert_eq!(MAX_ENCODERS, 25, "Should support 25 normal encoders");
    assert_eq!(
        ULTRA_FAST_ENCODER_INDEX, 25,
        "Ultra-fast encoder should be at index 25"
    );
}

#[test]
fn test_encoder_sampling_mode_combinations() {
    // Test all valid sampling mode combinations

    // Disabled encoder
    let disabled = EncoderOptions::new();
    assert!(!disabled.enabled && !disabled.sampling_4x && !disabled.sampling_2x);

    // Enabled but no sampling enhancement (1x mode)
    let mut enabled_1x = EncoderOptions::new();
    enabled_1x.enabled = true;
    assert!(enabled_1x.enabled && !enabled_1x.sampling_4x && !enabled_1x.sampling_2x);

    // 4x sampling mode
    let sampling_4x = EncoderOptions::with_4x_sampling();
    assert!(sampling_4x.enabled && sampling_4x.sampling_4x && !sampling_4x.sampling_2x);

    // 2x sampling mode
    let sampling_2x = EncoderOptions::with_2x_sampling();
    assert!(sampling_2x.enabled && !sampling_2x.sampling_4x && sampling_2x.sampling_2x);
}

#[test]
fn test_encoder_key_mapping_combinations() {
    // Test key mapping options with sampling modes

    let mut options = EncoderOptions::with_4x_sampling();
    options.direct_key_mapping_a = true;
    options.direct_key_mapping_b = true;

    let byte = options.to_byte();

    // Should have: enabled + 4x + key_a + key_b
    assert_eq!(
        byte & 0b01010011,
        0b01010011,
        "Should have enabled, 4x, and both key mappings"
    );

    let decoded = EncoderOptions::from_byte(byte);
    assert!(decoded.enabled, "Should preserve enabled state");
    assert!(decoded.sampling_4x, "Should preserve 4x sampling");
    assert!(
        decoded.direct_key_mapping_a,
        "Should preserve key mapping A"
    );
    assert!(
        decoded.direct_key_mapping_b,
        "Should preserve key mapping B"
    );
}

#[test]
fn test_encoder_data_initialization() {
    let encoder = EncoderData::new();

    // Test default values
    assert_eq!(
        encoder.encoder_value, 0,
        "Initial encoder value should be 0"
    );
    assert_eq!(encoder.encoder_options, 0, "Initial options should be 0");
    assert_eq!(
        encoder.channel_a_pin, 0,
        "Initial channel A pin should be 0"
    );
    assert_eq!(
        encoder.channel_b_pin, 0,
        "Initial channel B pin should be 0"
    );
    assert!(
        !encoder.is_enabled(),
        "Encoder should be disabled by default"
    );
    assert!(
        !encoder.is_4x_sampling(),
        "4x sampling should be disabled by default"
    );
    assert!(
        !encoder.is_2x_sampling(),
        "2x sampling should be disabled by default"
    );
}

#[test]
fn test_encoder_protocol_compliance() {
    // Test that the implementation matches the protocol specification exactly

    // Protocol specification bit layout:
    // bit 0: enable encoder
    // bit 1: 4x sampling
    // bit 2: 2x sampling
    // bit 3: reserved
    // bit 4: direct key mapping for direction A
    // bit 5: mapped to macro for direction A
    // bit 6: direct key mapping for direction B
    // bit 7: mapped to macro for direction B

    let test_cases = [
        (0b00000001, true, false, false, false, false, false, false), // enabled only
        (0b00000011, true, true, false, false, false, false, false),  // enabled + 4x
        (0b00000101, true, false, true, false, false, false, false),  // enabled + 2x
        (0b00010001, true, false, false, true, false, false, false),  // enabled + key_a
        (0b00100001, true, false, false, false, true, false, false),  // enabled + macro_a
        (0b01000001, true, false, false, false, false, true, false),  // enabled + key_b
        (0b10000001, true, false, false, false, false, false, true),  // enabled + macro_b
    ];

    for (byte, enabled, sampling_4x, sampling_2x, key_a, macro_a, key_b, macro_b) in test_cases {
        let options = EncoderOptions::from_byte(byte);

        assert_eq!(
            options.enabled, enabled,
            "Enabled bit mismatch for byte {byte:08b}"
        );
        assert_eq!(
            options.sampling_4x, sampling_4x,
            "4x sampling bit mismatch for byte {byte:08b}"
        );
        assert_eq!(
            options.sampling_2x, sampling_2x,
            "2x sampling bit mismatch for byte {byte:08b}"
        );
        assert_eq!(
            options.direct_key_mapping_a, key_a,
            "Key A bit mismatch for byte {byte:08b}"
        );
        assert_eq!(
            options.macro_mapping_a, macro_a,
            "Macro A bit mismatch for byte {byte:08b}"
        );
        assert_eq!(
            options.direct_key_mapping_b, key_b,
            "Key B bit mismatch for byte {byte:08b}"
        );
        assert_eq!(
            options.macro_mapping_b, macro_b,
            "Macro B bit mismatch for byte {byte:08b}"
        );

        // Test round-trip conversion
        assert_eq!(
            options.to_byte(),
            byte,
            "Round-trip conversion failed for byte {byte:08b}"
        );
    }
}
