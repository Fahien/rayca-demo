# Rayca-Demo

A simple demo application showcasing the Rayca rendering engine.

## Prerequisites

Ensure you have the following installed:
- [Rustup](https://rustup.rs/)
  ```console
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

For Android development:
- [Cargo XBuild](https://github.com/rust-mobile/xbuild)
  ```console
  cargo install cargo-xbuild
  ```
- LLVM
  ```console
  brew install llvm
  ```

## Running on Dektop

To run the demo, simply execute:

```console
cargo run
```

## Running on Android

To run the demo on an Android device, follow these steps:

1. Ensure you have connected your Android device via USB and [enabled USB debugging](https://developer.android.com/studio/debug/dev-options).
2. Ensure adb is installed and available in your PATH.
   ```console
   brew install android-commandlinetools
   ```
3. Ensure the device is recognized by running:
   ```console
   $ adb devices
   List of devices attached
   ab123c45        device
   ```
4. Build and run the demo on the device:
   ```console
   x run --device adb:ab123c45
   ```
