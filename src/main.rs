#![no_std]
#![no_main]

extern crate arduino_nano33iot as hal;
extern crate cortex_m;
extern crate panic_halt;
extern crate usb_device;
extern crate usbd_serial;

use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::Peripherals;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = hal::Pins::new(peripherals.PORT);
    let mut red_led = pins.led_sck.into_open_drain_output(&mut pins.port);

    let bus_allocator = hal::usb_allocator(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm,
        pins.usb_dp,
    );

    let mut serial = SerialPort::new(&bus_allocator);
    let mut bus = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x0000, 0x0000))
        .manufacturer("Alexander Gherardi")
        .product("ATEM Compatible Tally Light")
        .serial_number("TODO")
        .device_class(USB_CLASS_CDC)
        .build();
    loop {
        // Poll the usb bus
        if !bus.poll(&mut [&mut serial]) {
            continue;
        }

        // Buffer stores serial data
        let mut buf = [0u8; 64];

        match serial.read(&mut buf[..]) {
            Ok(_) => {}
            Err(UsbError::WouldBlock) => {}
            Err(_) => {}
        };

        match serial.write(&[0x3a, 0x29]) {
            Ok(_) => {}
            Err(UsbError::WouldBlock) => {}
            Err(_) => {}
        };
    }
}
