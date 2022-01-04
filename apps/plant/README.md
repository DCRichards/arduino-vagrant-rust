# Plant

> 🌱 An arduino-powered plant monitoring and hydration system.

## Structure

This project uses [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to separate device-dependent code from device-independent code. The crates are as follows:

* `app` - The application binary, containing all device-dependent code.
* `lib` - A package with functionality which can be used without a device. Unit tests are provided here and can be run independently on the host.

## USB Logging

For ease of debugging, a USB serial logger has been added. To read messages, plug in the device and run:

```sh
screen /dev/tty[name of device]
```

Some quirks:

* `cat` does not work for this for some reason, so you will need to `screen` to view serial output.
* The device name will vary by platform. On Mac it will likely be something like `tty.usbmodem14101` whereas on Linux it is likely to be `/dev/ttyACM0`.

## Setup

```sh
make deploy
```

## Tests

Run all unit tests on the host:

```sh
make test
```
