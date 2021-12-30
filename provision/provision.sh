echo "----- provisioning -----"

# Install required libraries - linux-image-extra-virtual is especially important as it
# adds the required kernel modules for serial over USB access.
apt-get update -y && \
  apt-get install -y gcc-arm-none-eabi binutils-arm-none-eabi gcc-multilib \
  usbutils libudev-dev build-essential libusb-1.0-0-dev linux-image-extra-virtual && \
	apt-get -y autoremove

su vagrant

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustup install nightly
rustup default nightly
rustup target add thumbv6m-none-eabi

# $PATH
touch ~/.bashrc
echo "export PATH=\$PATH:/usr/local/bin:\$HOME/.cargo/bin" >> ~/.bashrc

# Arduino CLI
curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | BINDIR=/usr/local/bin sh
arduino-cli core install arduino:samd

su

# Add udev rules for ensuring USB access.
cat <<EOF > /etc/udev/rules.d/20-hw1.rules
SUBSYSTEMS=="usb", ATTRS{idVendor}=="2341", TAG+="uaccess", TAG+="udev-acl"
EOF
udevadm trigger
udevadm control --reload-rules
usermod -a -G plugdev vagrant
usermod -a -G dialout vagrant

echo "----- provisioned -----"
