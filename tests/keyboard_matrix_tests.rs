//! Comprehensive tests for keyboard matrix functionality

use pokeys_lib::keyboard_matrix::MatrixKeyboard;

#[test]
fn test_matrix_keyboard_new() {
    let kb = MatrixKeyboard::new();

    assert_eq!(kb.configuration, 0);
    assert_eq!(kb.width, 0);
    assert_eq!(kb.height, 0);
    assert_eq!(kb.scanning_decimation, 0);
    assert_eq!(kb.column_pins.len(), 8);
    assert_eq!(kb.row_pins.len(), 16);
    assert_eq!(kb.macro_mapping_options.len(), 128);
    assert_eq!(kb.key_mapping_key_code.len(), 128);
    assert_eq!(kb.key_mapping_key_modifier.len(), 128);
    assert_eq!(kb.key_mapping_triggered_key.len(), 128);
    assert_eq!(kb.key_mapping_key_code_up.len(), 128);
    assert_eq!(kb.key_mapping_key_modifier_up.len(), 128);
    assert_eq!(kb.key_values.len(), 128);

    // All arrays should be initialized to zero
    assert!(kb.column_pins.iter().all(|&x| x == 0));
    assert!(kb.row_pins.iter().all(|&x| x == 0));
    assert!(kb.macro_mapping_options.iter().all(|&x| x == 0));
    assert!(kb.key_values.iter().all(|&x| x == 0));
}

#[test]
fn test_matrix_keyboard_default() {
    let kb = MatrixKeyboard::default();
    let kb_new = MatrixKeyboard::new();

    assert_eq!(kb.configuration, kb_new.configuration);
    assert_eq!(kb.width, kb_new.width);
    assert_eq!(kb.height, kb_new.height);
}

#[test]
fn test_is_enabled() {
    let mut kb = MatrixKeyboard::new();

    // Initially disabled
    assert!(!kb.is_enabled());

    // Enable it
    kb.configuration = 1;
    assert!(kb.is_enabled());

    // Any non-zero value should enable it
    kb.configuration = 255;
    assert!(kb.is_enabled());

    // Back to disabled
    kb.configuration = 0;
    assert!(!kb.is_enabled());
}

#[test]
fn test_get_key_state_bounds_checking() {
    let mut kb = MatrixKeyboard::new();
    kb.width = 4;
    kb.height = 4;

    // Valid positions should return false initially
    assert!(!kb.get_key_state(0, 0));
    assert!(!kb.get_key_state(3, 3));

    // Out of bounds positions should return false
    assert!(!kb.get_key_state(4, 0)); // Row out of bounds
    assert!(!kb.get_key_state(0, 4)); // Column out of bounds
    assert!(!kb.get_key_state(4, 4)); // Both out of bounds
    assert!(!kb.get_key_state(100, 100)); // Way out of bounds
}

#[test]
fn test_get_key_state_index_calculation() {
    let mut kb = MatrixKeyboard::new();
    kb.width = 3;
    kb.height = 3;

    // Test the key index calculation: row * 8 + col (protocol uses 8-column layout)
    // For a 3x3 matrix with 8-column internal layout:
    // (0,0) = 0*8+0 = 0, (0,1) = 0*8+1 = 1, (0,2) = 0*8+2 = 2
    // (1,0) = 1*8+0 = 8, (1,1) = 1*8+1 = 9, (1,2) = 1*8+2 = 10
    // (2,0) = 2*8+0 = 16, (2,1) = 2*8+1 = 17, (2,2) = 2*8+2 = 18

    // Set specific keys
    kb.key_values[0] = 1; // (0,0)
    kb.key_values[1] = 1; // (0,1)
    kb.key_values[9] = 1; // (1,1)
    kb.key_values[18] = 1; // (2,2)

    // Test the set keys
    assert!(kb.get_key_state(0, 0));
    assert!(kb.get_key_state(0, 1));
    assert!(kb.get_key_state(1, 1));
    assert!(kb.get_key_state(2, 2));

    // Test unset keys
    assert!(!kb.get_key_state(0, 2));
    assert!(!kb.get_key_state(1, 0));
    assert!(!kb.get_key_state(1, 2));
    assert!(!kb.get_key_state(2, 0));
    assert!(!kb.get_key_state(2, 1));
}

#[test]
fn test_different_matrix_sizes() {
    // Test 1x1 matrix
    let mut kb1 = MatrixKeyboard::new();
    kb1.width = 1;
    kb1.height = 1;
    kb1.key_values[0] = 1;

    assert!(kb1.get_key_state(0, 0));
    assert!(!kb1.get_key_state(0, 1)); // Out of bounds
    assert!(!kb1.get_key_state(1, 0)); // Out of bounds

    // Test 8x16 matrix (maximum size)
    let mut kb2 = MatrixKeyboard::new();
    kb2.width = 8;
    kb2.height = 16;

    // Test corners
    kb2.key_values[0] = 1; // (0,0) = 0*8 + 0 = 0
    kb2.key_values[7] = 1; // (0,7) = 0*8 + 7 = 7
    kb2.key_values[120] = 1; // (15,0) = 15*8 + 0 = 120
    kb2.key_values[127] = 1; // (15,7) = 15*8 + 7 = 127

    assert!(kb2.get_key_state(0, 0));
    assert!(kb2.get_key_state(0, 7));
    assert!(kb2.get_key_state(15, 0));
    assert!(kb2.get_key_state(15, 7));

    // Test out of bounds
    assert!(!kb2.get_key_state(16, 0)); // Row out of bounds
    assert!(!kb2.get_key_state(0, 8)); // Column out of bounds
}

#[test]
fn test_key_values_array_bounds() {
    let mut kb = MatrixKeyboard::new();
    kb.width = 8;
    kb.height = 16; // This gives us 128 keys total

    // The last valid key should be at index 127
    kb.key_values[127] = 1;
    assert!(kb.get_key_state(15, 7)); // (15*8 + 7 = 127)

    // Test that we don't access beyond the array
    // Even if we somehow had a larger matrix, get_key_state should handle it gracefully
    kb.width = 10; // This would theoretically give us more keys
    kb.height = 20; // But we're still limited by the 128-element array

    // This should not panic and should return false for out-of-array indices
    assert!(!kb.get_key_state(19, 9)); // This would be index 19*10 + 9 = 199, beyond array
}

#[test]
fn test_zero_size_matrix() {
    let kb = MatrixKeyboard::new();
    // Default width and height are 0

    // Any access should return false
    assert!(!kb.get_key_state(0, 0));
    assert!(!kb.get_key_state(1, 1));
}

#[test]
fn test_pin_arrays() {
    let mut kb = MatrixKeyboard::new();

    // Test column pins (max 8)
    for i in 0..8 {
        kb.column_pins[i] = (i + 1) as u8;
    }

    for i in 0..8 {
        assert_eq!(kb.column_pins[i], (i + 1) as u8);
    }

    // Test row pins (max 16)
    for i in 0..16 {
        kb.row_pins[i] = (i + 10) as u8;
    }

    for i in 0..16 {
        assert_eq!(kb.row_pins[i], (i + 10) as u8);
    }
}

#[test]
fn test_key_mapping_arrays() {
    let mut kb = MatrixKeyboard::new();

    // Test that all key mapping arrays have the correct size
    assert_eq!(kb.key_mapping_key_code.len(), 128);
    assert_eq!(kb.key_mapping_key_modifier.len(), 128);
    assert_eq!(kb.key_mapping_triggered_key.len(), 128);
    assert_eq!(kb.key_mapping_key_code_up.len(), 128);
    assert_eq!(kb.key_mapping_key_modifier_up.len(), 128);

    // Test that we can write to all positions
    for i in 0..128 {
        kb.key_mapping_key_code[i] = i as u8;
        kb.key_mapping_key_modifier[i] = (i % 256) as u8;
    }

    // Verify the values
    for i in 0..128 {
        assert_eq!(kb.key_mapping_key_code[i], i as u8);
        assert_eq!(kb.key_mapping_key_modifier[i], (i % 256) as u8);
    }
}

#[test]
fn test_realistic_keyboard_scenarios() {
    // Test a realistic 4x4 keypad
    let mut kb = MatrixKeyboard::new();
    kb.configuration = 1;
    kb.width = 4;
    kb.height = 4;

    // Simulate pressing keys in a pattern
    // Press keys: (0,0), (1,1), (2,2), (3,3) - diagonal
    kb.key_values[0] = 1; // (0,0) = 0*8 + 0 = 0
    kb.key_values[9] = 1; // (1,1) = 1*8 + 1 = 9
    kb.key_values[18] = 1; // (2,2) = 2*8 + 2 = 18
    kb.key_values[27] = 1; // (3,3) = 3*8 + 3 = 27

    assert!(kb.is_enabled());
    assert!(kb.get_key_state(0, 0));
    assert!(kb.get_key_state(1, 1));
    assert!(kb.get_key_state(2, 2));
    assert!(kb.get_key_state(3, 3));

    // Test that other keys are not pressed
    assert!(!kb.get_key_state(0, 1));
    assert!(!kb.get_key_state(1, 0));
    assert!(!kb.get_key_state(2, 1));
    assert!(!kb.get_key_state(3, 2));
}

#[test]
fn test_full_keyboard_press() {
    // Test pressing all keys in a small matrix
    let mut kb = MatrixKeyboard::new();
    kb.width = 2;
    kb.height = 2;

    // Press all 4 keys using 8-column layout
    kb.key_values[0] = 1; // (0,0) = 0*8 + 0 = 0
    kb.key_values[1] = 1; // (0,1) = 0*8 + 1 = 1
    kb.key_values[8] = 1; // (1,0) = 1*8 + 0 = 8
    kb.key_values[9] = 1; // (1,1) = 1*8 + 1 = 9

    // Verify all keys are pressed
    assert!(kb.get_key_state(0, 0)); // index 0
    assert!(kb.get_key_state(0, 1)); // index 1
    assert!(kb.get_key_state(1, 0)); // index 8
    assert!(kb.get_key_state(1, 1)); // index 9
}
