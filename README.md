# MultiTherm
This repository contains the hardware, firmware, and drivers for MultiTherm.

MultiTherm allows you to log data from a few [tiny I2C temperature sensors](https://www.ti.com/lit/ds/symlink/tmp117.pdf) via your computer's USB port. A few could range from 1-4 or more.

Each temperature sensor is on a flexible 200mm probe ribbon; route them in free space however you like, or even trim their ribbons to length. An adhesive sticker provides thermal coupling to your device under test.

MultiTherm can be used as a lower-cost substitute for a resistance temperature detector (RTD) and data acquisition interface (DAQ) while still featuring NIST-traceable calibration accurate within ±0.1°C (between –20°C to 50°C) and 0.0078°C read-out resolution.

Working temperature range is –55 °C to 150 °C.

MultiTherm will be made available for purchase at a later date.

## Applications
- Optical bench temperature compensation
- Bulk material conductivity measurement
- Power electronics testing and characterization
- Chemical synthesis
- Bioreactors, polymerase chain reaction
- Wearable devices

## Project structure
- ESP32C6 firmware: [mt-fw](./mt-fw)
- PCB design files: [mt-hw](./mt-hw)
- Userspace serial-to-npyz driver: [mt-relay](./mt-relay)

## License
[GPLv3](./LICENSE)

## Copyright
[Paul Hansel](https://paulhansel.com), 2024

## Thanks to:
- [Rust on ESP community](https://github.com/esp-rs)
- [KiCAD developers](https://www.kicad.org/sponsors/sponsors/)
- [Rust Embedded Working Group](https://blog.rust-embedded.org/)

