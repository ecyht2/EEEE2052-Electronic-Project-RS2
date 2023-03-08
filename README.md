# EEEE2052-Electronic-Project-rs2

This repo contains STM32 code for Electronic Project of EEEE2052 module. It uses the [stm32l4xx-hal](https://github.com/stm32-rs/stm32l4xx-hal) crate. It's based on the [Knurling app template](https://github.com/knurling-rs/app-template).

This is one of the three version of the same code:
- [STM32 Cube IDE (C)](https://github.com/ecyht2/EEEE2052-Electronic-Project/)
- [STM32-HAL](https://github.com/ecyht2/)
- [stm32l4xx-hal](https://github.com/ecyht2/)

## Quickstart
- [Install Rust](https://www.rust-lang.org/tools/install).
- Install the compilation target for your MCU. Eg run `rustup target add thumbv7em-none-eabihf`. You'll need to change the last part if using a Cortex-M0, Cortex-M33, (Eg Stm32G0 or L5 respectively) or if you don't want to use hardware floats.
- Install flash and debug tools: `cargo install flip-link`, `cargo install probe-run`.
- Clone this repo: `git clone https://github.com/ecyht2/electronics-project2`
- Connect your device. Run `cargo run --release` to compile and flash.
