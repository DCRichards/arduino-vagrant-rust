# Plant

> 🌱 An arduino-powered plant monitoring and hydration system.

## Structure

This project uses [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to separate device-dependent code from device-independent code. The crates are as follows:

* `app` - The application binary, containing all device-dependent code.
* `lib` - A package with functionality which can be used without a device. Unit tests are provided here and can be run independently on the host.

## Setup

```sh
make deploy
```

## Tests

Run all unit tests on the host:

```sh
make test
```
