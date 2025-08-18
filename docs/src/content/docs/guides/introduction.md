---
title: Introduction
description: Introduction to the PoKeys Core Library
---

# PoKeys Core Library

A pure Rust implementation of the PoKeysLib for controlling PoKeys devices. This is the **core library** that provides all fundamental device communication and control functionality for the PoKeys ecosystem.

## 🚀 Performance Breakthrough: Dual Optimization System

**Revolutionary dual optimization system provides massive performance improvements**:

### Bulk Operations Optimization
- **Before**: 110 individual commands, 14.44ms configuration time
- **After**: 2 bulk commands, 513µs configuration time
- **Result**: 96.4% time reduction, **28x faster pin configuration**

### Single Enumeration Optimization
- **Before**: Multiple 5-second device enumerations per sync
- **After**: Single enumeration, cached results reused
- **Result**: 65% faster device discovery, **3x faster multi-device sync**

### Encoder Pin Numbering Fix
- **Fixed**: Encoder pin numbering conversion (1-based config ↔ 0-based protocol)
- **Result**: Correct encoder pin assignments in vendor tools
- **Impact**: Reliable encoder configuration and monitoring

## ✨ Core Features

### Device Connectivity
- **USB Devices**: Full support for USB-connected PoKeys devices
- **Network Devices**: Discovery and connection to network-enabled devices
- **Auto-Detection**: Intelligent connection type detection
- **Multi-Device**: Concurrent management of multiple devices

### Digital & Analog I/O
- **Digital I/O**: Pin configuration and digital input/output operations
- **Analog I/O**: Multi-channel analog input with configurable reference voltage
- **Pin Functions**: Digital input/output, analog input, PWM, encoder, counter, keyboard matrix
- **Bulk Operations**: Optimized bulk pin configuration and state reading

### Advanced Control Systems
- **PWM Control**: Multiple PWM channels with configurable frequency and duty cycle
- **Encoder Support**: Quadrature encoder reading with 4x/2x sampling modes, position and velocity tracking
- **Pulse Engine v2**: Stepper motor control with advanced pulse generation
- **Matrix Operations**: Matrix keyboard scanning and LED matrix control

### Communication Protocols
- **SPI**: Full SPI master support with multiple chip select pins
- **I2C**: I2C master operations with device scanning
- **1-Wire**: 1-Wire protocol support for temperature sensors
- **CAN Bus**: CAN message transmission and reception
- **UART**: Serial communication support

### Display & Interface Support
- **LCD Display**: Text LCD display control and management
- **Seven-Segment**: Built-in character mapping and display utilities

### Sensor Integration
- **EasySensors**: Integrated sensor support and data acquisition
- **Real-Time Clock**: RTC operations and time synchronization
- **Temperature Sensors**: 1-Wire temperature sensor support

### Safety & Reliability
- **Device Models**: Comprehensive pin capability validation and safety checks
- **Error Handling**: Detailed error types with context and recovery suggestions
- **Thread Safety**: Safe concurrent access to device resources
- **Failsafe Settings**: Configurable failsafe behavior for critical applications
- **SPI Pin Reservation**: Hardware constraint enforcement prevents conflicts
