#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate arduino_nano33iot as hal;
extern crate alloc;

use core::convert::Infallible;

use {
    hal::clock::GenericClockController,
    hal::delay::Delay,
    hal::entry,
    hal::pac::{interrupt, CorePeripherals, Peripherals},
    hal::prelude::*,
    hal::usb::usb_device::bus::UsbBusAllocator,
    hal::UsbBus,
};

use {
    cortex_m::peripheral::NVIC,
    onewire::*,
    usb_device::prelude::*,
    usbd_serial::{SerialPort, USB_CLASS_CDC},
    alloc_cortex_m::CortexMHeap,
};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

#[entry]
fn main() -> ! {
    // Initialise heap
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 1024);
    };

    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut pins = hal::Pins::new(peripherals.PORT);
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(hal::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            pins.usb_dm,
            pins.usb_dp,
            &mut pins.port,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(&bus_allocator));
        USB_BUS = Some(
            // vendor id, product id
            UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x2341, 0x8057))
                .manufacturer("Arduino LLC")
                .product("Arduino NANO 33 IoT")
                .serial_number("4197FF4750535131332E3120FF0B1541")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    let mut led = pins.led_sck.into_open_drain_output(&mut pins.port);
    let mut pin_d2 = pins.d2.into_readable_open_drain_output(&mut pins.port);
    let mut one_wire = OneWire::new(&mut pin_d2, false);

    match one_wire.reset(&mut delay) {
        Ok(_) => {}
        Err(_) => {
            log_str("error: unable to reset, likely a missing pullup or line error\r\n");
            loop {}
        }
    };

    let mut search = DeviceSearch::new();
    loop {
        while let Some(device) = one_wire.search_next(&mut search, &mut delay).unwrap() {
            match device.address[0] {
                ds18b20::FAMILY_CODE => {
                    led.set_high().unwrap();
                    let sensor = DS18B20::new::<Infallible>(device).unwrap();
                    let resolution = sensor
                        .measure_temperature(&mut one_wire, &mut delay)
                        .unwrap();
                    delay.delay_ms(resolution.time_ms());
                    let raw = sensor.read_temperature(&mut one_wire, &mut delay).unwrap();
                    let (integer, fraction) = ds18b20::split_temp(raw);
                    let temperature: f32 = integer as f32 + fraction as f32/10000_f32;
                    log(alloc::format!("Current temperature: {}°C\r\n", temperature).as_bytes());
                }
                _ => {
                    log_str("unknown device\r\n");
                }
            }
        }
    }
}

fn log_str(msg: &str) {
    log(msg.as_bytes());
}

fn log(bytes: &[u8]) {
    cortex_m::interrupt::free(|_| unsafe {
        USB_BUS.as_mut().map(|_| {
            USB_SERIAL.as_mut().map(|serial| {
                // Skip errors so we can continue the program
                let _ = serial.write(bytes);
            });
        })
    });
}

#[interrupt]
unsafe fn USB() {
    poll_usb();
}

unsafe fn poll_usb() {
    USB_BUS.as_mut().map(|usb_dev| {
        USB_SERIAL.as_mut().map(|serial| {
            usb_dev.poll(&mut [serial]);
            let mut buf = [0u8; 16];
            let _ = serial.read(&mut buf);
            // log(&buf);
        });
    });
}
