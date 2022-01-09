#!/bin/bash

curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh

./bin/arduino-cli core install arduino:samd

cargo install cargo-binutils

rustup component add llvm-tools-preview