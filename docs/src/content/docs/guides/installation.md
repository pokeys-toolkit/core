---
title: Installation
description: How to install the PoKeys Core Library
---

# Installation

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pokeys-lib = "0.5.0"
```

Or install from git:

```toml
[dependencies]
pokeys-lib = { git = "https://github.com/pokeys-toolkit/core" }
```

## System Requirements

- **Rust**: 1.70 or later
- **Operating Systems**: Windows, macOS, Linux
- **Hardware**: PoKeys devices (USB or network-connected)

## Supported Devices

- **PoKeys55 v3** - 55-pin development board
- **PoKeys56U** - USB-enabled 56-pin device (55 usable pins, 31 CS-capable)
- **PoKeys56E** - Ethernet-enabled 56-pin device (55 usable pins, 31 CS-capable)
- **PoKeys57U** - Advanced USB device (57 usable pins, 33 CS-capable)
- **PoKeys57E** - Ethernet-enabled industrial device (57 usable pins, 33 CS-capable)
- **PoKeys58EU** - Ethernet-enabled device with extended features
- **Custom Devices** - Extensible model system for custom hardware

## Verification

After installation, verify everything works:

```bash
cargo check
```

You're now ready to start using the PoKeys Core Library!
