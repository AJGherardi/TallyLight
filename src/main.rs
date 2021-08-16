#![no_std]
#![no_main]

extern crate arduino_nano33iot as hal;
extern crate cortex_m;
extern crate hex;
extern crate panic_halt;
extern crate usb_device;
extern crate usbd_serial;

use core::str;
use embedded_hal::PwmPin;
use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::pwm::{Channel, Pwm0, Pwm5};
use hex::ToHex;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    // Get peripherals and clocks from hal
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    // Get pins from peripherals
    let mut pins = hal::Pins::new(peripherals.PORT);

    // Get led from pins
    let mut _led_prev = pins.a3.into_function_f(&mut pins.port);
    let mut _led_prog = pins.a2.into_function_f(&mut pins.port);
    let mut _led_prog_fr = pins.d9.into_function_f(&mut pins.port);

    // Get PWM
    let gclk0 = clocks.gclk0();
    let mut pwm = Pwm0::new(
        &clocks.tcc0_tcc1(&gclk0).unwrap(),
        1.khz(),
        peripherals.TCC0,
        &mut peripherals.PM,
    );

    // Enable PWM a2: _3; a3: _2
    let max_duty = pwm.get_max_duty();
    pwm.enable(Channel::_3);
    pwm.enable(Channel::_2);
    pwm.enable(Channel::_1);
    let led_prev = Channel::_2;
    let led_prog = Channel::_3;
    let led_prog_fr = Channel::_1;

    // Get bus allocator
    let bus_allocator = hal::usb_allocator(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm,
        pins.usb_dp,
    );

    // Get serial number from hal and encode it into a hex string
    let serial_number = hal::serial_number();
    let mut serial_hex = [0u8; 32];
    hex::encode_to_slice(serial_number, &mut serial_hex);
    let serial_str = match str::from_utf8(&serial_hex) {
        Ok(v) => v,
        Err(_) => "NA",
    };

    // Create serial and bus using allocator
    let mut serial = SerialPort::new(&bus_allocator);
    let mut bus = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x0000, 0x0000))
        .manufacturer("Alexander Gherardi")
        .product("ATEM Compatible Tally Light")
        .serial_number(serial_str)
        .device_class(USB_CLASS_CDC)
        .build();

    loop {
        // Poll the usb bus
        if !bus.poll(&mut [&mut serial]) {
            continue;
        }

        // Buffer stores serial data
        let mut buf = [0u8; 64];

        // Read from the serial port
        match serial.read(&mut buf[..]) {
            Ok(count) => {
                // Parse buffer into a string
                let s = match str::from_utf8(&buf[0..count]) {
                    Ok(v) => v,
                    Err(_) => "",
                };

                // Match string to desired state
                match s {
                    "prev_on\n" => pwm.set_duty(led_prev, max_duty / 4),
                    "prev_off\n" => pwm.set_duty(led_prev, 0),
                    "prog_on\n" => {
                        pwm.set_duty(led_prog, max_duty);
                        pwm.set_duty(led_prog_fr, max_duty);
                    }
                    "prog_off\n" => {
                        pwm.set_duty(led_prog, 0);
                        pwm.set_duty(led_prog_fr, 0);
                    }
                    _ => {}
                }
            }
            Err(UsbError::WouldBlock) => {}
            Err(_) => {}
        };
    }
}
