use {
    cortex_m::peripheral::NVIC,
    hal::clock::GenericClockController,
    hal::gpio::{Floating, Input, Pa24, Pa25, Port},
    hal::pac::{interrupt, PM, USB},
    hal::usb::usb_device::bus::UsbBusAllocator,
    hal::usb_allocator,
    hal::UsbBus,
    usb_device::prelude::*,
    usbd_serial::{SerialPort, USB_CLASS_CDC},
};

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

/// Represents a logger which logs over USB Serial.
pub struct USBLogger {}

impl USBLogger {
    /// Get the log instance
    pub fn new(
        usb: USB,
        clocks: &mut GenericClockController,
        pm: &mut PM,
        dm: Pa24<Input<Floating>>,
        dp: Pa25<Input<Floating>>,
        port: &mut Port,
        nvic: &mut hal::pac::NVIC,
    ) -> Self {
        let bus_allocator = unsafe {
            USB_ALLOCATOR = Some(usb_allocator(usb, clocks, pm, dm, dp, port));
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
            nvic.set_priority(interrupt::USB, 1);
            NVIC::unmask(interrupt::USB);
        }

        USBLogger {}
    }

    /// Log writes a log entry
    pub fn log(&self, bytes: &[u8]) {
        cortex_m::interrupt::free(|_| unsafe {
            USB_BUS.as_mut().map(|_| {
                USB_SERIAL.as_mut().map(|serial| {
                    // Skip errors so we can continue the program
                    let _ = serial.write(bytes);
                });
            })
        });
    }
}

#[interrupt]
unsafe fn USB() {
    USB_BUS.as_mut().map(|usb_dev| {
        USB_SERIAL.as_mut().map(|serial| {
            usb_dev.poll(&mut [serial]);
            let mut buf = [0u8; 16];
            let _ = serial.read(&mut buf);
            // log(&buf);
        });
    });
}
