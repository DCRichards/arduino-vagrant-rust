#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;
extern crate arduino_nano33iot as hal;

use core::convert::Infallible;

use {
    hal::adc::Adc,
    hal::clock::GenericClockController,
    hal::delay::Delay,
    hal::entry,
    hal::pac::{interrupt, CorePeripherals, Peripherals},
    hal::prelude::*,
    hal::usb::usb_device::bus::UsbBusAllocator,
    hal::UsbBus,
};

use {
    alloc_cortex_m::CortexMHeap,
    cortex_m::peripheral::NVIC,
    onewire::*,
    usb_device::prelude::*,
    usbd_serial::{SerialPort, USB_CLASS_CDC},
};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

#[entry]
fn main() -> ! {
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
    let mut temp_sensor = pins.d2.into_readable_open_drain_output(&mut pins.port);
    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    // adc.reference(hal::pac::adc::refctrl::REFSEL_A::AREFA);
    let mut moisture_sensor = pins.a0.into_function_b(&mut pins.port);

    // TODO: Move this into its own struct and methods.
    let mut one_wire = OneWire::new(&mut temp_sensor, false);
    one_wire.reset(&mut delay).unwrap();

    let mut search = DeviceSearch::new_for_family(ds18b20::FAMILY_CODE);
    match one_wire.search_next(&mut search, &mut delay).unwrap() {
        None => {
            log("No temperature sensor found\r\n".as_bytes());
            loop {}
        }
        Some(device) => {
            led.set_high().unwrap();
            let sensor = DS18B20::new::<Infallible>(device).unwrap();

            loop {
                let resolution = sensor
                    .measure_temperature(&mut one_wire, &mut delay)
                    .unwrap();
                delay.delay_ms(resolution.time_ms());
                let raw = sensor.read_temperature(&mut one_wire, &mut delay).unwrap();
                let (integer, fraction) = ds18b20::split_temp(raw);
                let temperature: f32 = integer as f32 + fraction as f32 / 10000_f32;

                // TODO: also move this into its own struct.
                let moisture: u16 = adc.read(&mut moisture_sensor).unwrap();
                log(alloc::format!(
                    "Temperature: {}°C\r\nMoisture: {}\r\n",
                    temperature,
                    moisture
                )
                .as_bytes());
            }
        }
    };
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
