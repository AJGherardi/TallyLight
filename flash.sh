#!/bin/bash

# Handels flashing of the device

# Build program
cargo build --release

# Package program into a bin file
rust-objcopy -O binary target/thumbv6m-none-eabi/release/tally target/bin.bin

# Flash the device
arduino-cli upload -i target/bin.bin -b arduino:samd:nano_33_iot -p /dev/ttyACM0