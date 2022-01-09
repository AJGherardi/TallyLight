#!/bin/bash

cargo build --release

rust-objcopy -O binary target/thumbv6m-none-eabi/release/tally target/tally.bin

./bin/arduino-cli upload -i target/tally.bin -b arduino:samd:nano_33_iot -p /dev/ttyACM0