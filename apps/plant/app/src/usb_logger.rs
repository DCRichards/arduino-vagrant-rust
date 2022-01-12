use {
    bsp::hal::clock::GenericClockController,
    bsp::hal::usb::UsbBus,
    bsp::pac::{interrupt, PM, USB},
    bsp::usb_allocator,
    core::fmt::{Error, Write},
    core::result::Result,
    cortex_m::interrupt::free,
    cortex_m::peripheral::NVIC,
    usb_device::bus::UsbBusAllocator,
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
        dm: bsp::UsbDm,
        dp: bsp::UsbDp,
        nvic: &mut bsp::pac::NVIC,
    ) -> Self {
        let bus_allocator = unsafe {
            USB_ALLOCATOR = Some(usb_allocator(usb, clocks, pm, dm, dp));
            USB_ALLOCATOR.as_ref().unwrap()
        };

        unsafe {
            USB_SERIAL = Some(SerialPort::new(bus_allocator));
            USB_BUS = Some(
                // vendor id, product id
                UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x2341, 0x8057))
                    .manufacturer("Arduino LLC")
                    .product("Arduino NANO 33 IoT")
                    .serial_number("4197FF4750535131332E3120FF0B1541")
                    .device_class(USB_CLASS_CDC)
                    .build(),
            );
            nvic.set_priority(interrupt::USB, 1);
            // Enable the interrupt in the NVIC.
            NVIC::unmask(interrupt::USB);
        }

        USBLogger {}
    }

    /// Log writes a log entry
    pub fn log(&self, s: &str) {
        free(|_| unsafe {
            USB_BUS.as_mut().map(|_| {
                if let Some(serial) = USB_SERIAL.as_mut() {
                    // Skip errors so we can continue the program
                    let _ = serial.write(s.as_bytes());
                }
            })
        });
    }
}

impl Write for USBLogger {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.log(s);
        Ok(())
    }
}

#[interrupt]
unsafe fn USB() {
    if let Some(usb_dev) = USB_BUS.as_mut() {
        if let Some(serial) = USB_SERIAL.as_mut() {
            usb_dev.poll(&mut [serial]);
            let mut buf = [0u8; 16];
            let _ = serial.read(&mut buf);
            // let _ = serial.write(&buf);
        }
    }
}
