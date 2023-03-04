# Doppler Radar Speed Detector

This containts code for a doppler radar based speed detection system. It uses an STM32L476RG MCU to capture and process the measured spped coming from the doppler radar.

This is based on the [STM32-HAL](https://github.com/David-OConnor/stm32-hal) template.

# Quickstart
- [Install Rust](https://www.rust-lang.org/tools/install).
- Install the compilation target for your MCU. Eg run `rustup target add thumbv7em-none-eabihf`. You'll need to change the last part if using a Cortex-M0, Cortex-M33, (Eg Stm32G0 or L5 respectively) or if you don't want to use hardware floats.
- Install flash and debug tools: `cargo install flip-link`, `cargo install probe-run`.
- Clone this repo: `git clone https://github.com/ecyht2/electronics-project2`
- Connect your device. Run `cargo run --release` to compile and flash.
