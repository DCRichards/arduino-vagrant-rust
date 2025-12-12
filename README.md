# arduino-vagrant-rust

> A Vagrant box for Arduino development with Rust.

## Setup


1. Initialise box:

   ```shell
   vagrant up
   ```

1. Add USB device filter to VirtualBox

   Head to Settings > USB in VirtualBox and add your board. Remember to do this with and without bootloader mode as this changes some of the USB device properties. For example, you should now see:

   * Arduino LLC Arduino NONA WLAN [0200]
   * Arduino LLC Arduino NANO 33 IoT [0010]

1. ssh in to your Vagrant box

    ```shell
    vagrant ssh
    ```

1. Check you are able to access the device. Use the following to check your board is connected and accessible. If you have connected your device after starting the vagrant machine, try restarting. Otherwise, head into VirtualBox and check there is a device filter for the currently connected Arduino then halt and up again.

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

   If you are using a board other than the Nano 33 IOT then find your board [here](https://github.com/atsamd-rs/atsamd) or elsewhere and duplicate the example project and replace the dependency and code to use the correct Board Support Crate.

