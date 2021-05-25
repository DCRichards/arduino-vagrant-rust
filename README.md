# arduino-vagrant-rust

> A Vagrant box for Arduino development with Rust.

## Setup

1. Simply bring up and ssh in:

   ```shell
   vagrant up
   vagrant ssh
   ```

1. Check you are able to access the device. Use the following to check your board is connected and accessible. If you have connected your device after starting the vagrant machine, try restarting. Otherwise, head into VirtualBox and add a device filter for the currently connected Arduino then halt and up again.

   ```shell
   usb-devices
   ls /dev/tty/ACM*
   arduino-cli board list
   ```

1. Test by running the example. You may need to press the reset button on your board before uploading.

   ```shell
   cd apps/blinky
   cargo build --release
   arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/blinky target/blinky.bin
   arduino-cli upload -i target/blinky.bin -b arduino:samd:nano_33_iot -p /dev/ttyACM0
   ```

   If you are using a board other than the Nano 33 IOT then find your board [here](https://github.com/atsamd-rs/atsamd) or elsewhere and duplicate the example project and replace the dependency and code to use the correct BSP.


## Inspiration

* [smart-home](https://github.com/olback/smart-home)
