#![no_std]
#![no_main]

extern crate cortex_m;
extern crate hex;
extern crate panic_halt;
extern crate usb_device;
extern crate usbd_serial;

use arduino_nano33iot as bsp;
use bsp::entry;
use bsp::hal;

use bsp::hal::gpio::v2::*;

use core::str;
use hal::clock::GenericClockController;
use hal::pac::Peripherals;
use hal::prelude::*;
use hal::pwm::{Channel, Pwm0};
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
    let pins = bsp::Pins::new(peripherals.PORT);

    // Get led from pins
    let _led_prev: Pin<_, AlternateF> = pins.a2.into_mode();
    let _led_prog: Pin<_, AlternateF> = pins.a3.into_mode();
    let _led_prog_fr: Pin<_, AlternateE> = pins.d5.into_mode();

    // Get PWM
    let gclk0 = clocks.gclk0();
    let mut pwm = Pwm0::new(
        &clocks.tcc0_tcc1(&gclk0).unwrap(),
        1.khz(),
        peripherals.TCC0,
        &mut peripherals.PM,
    );

    // Enable PWM a2: _3; a3: _2; d5: _1
    let max_duty = pwm.get_max_duty();
    pwm.enable(Channel::_2);
    pwm.enable(Channel::_3);
    pwm.enable(Channel::_1);
    let led_prev = Channel::_3;
    let led_prog = Channel::_2;
    let led_prog_fr = Channel::_1;

    // Get bus allocator
    let bus_allocator = bsp::usb_allocator(
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

                // Split string 
                let mut split = s.split("|");

                // Get desired light choice and lightness
                let light = split.nth(0).unwrap_or("");
                let lightness_str = split.nth(0).unwrap_or("0");
                let lightness: u32 = lightness_str.parse().unwrap_or(0);                    

                // Select light and set lightness
                match light {
                    "p" => pwm.set_duty(led_prev, max_duty / 100 * lightness),
                    "P" => {
                        pwm.set_duty(led_prog, max_duty / 100 * lightness);
                        pwm.set_duty(led_prog_fr, max_duty / 100 * lightness);
                    },
                    _ => {}
                }

            }
            Err(UsbError::WouldBlock) => {}
            Err(_) => {}
        };
    }
}
