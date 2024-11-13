# Roadmap
[X] Complete initial requirements document
[X] Publish PCB schematic and layout (GPLv3)
[X] Publish firmware (GPLv3)
[X] Publish userspace driver (GPLv3)
[ ] Project write-up
[ ] Ship integrated product (USB hub PCB, ribbons, documentation) to market

## Hardware
[X] Ribbon probe PCB schematic
[X] Ribbon probe PCB layout
[X] Fabricate and validate probe ribbon PCB (Rev 1.0)
[ ] Fabricate and validate probe ribbon PCB (Rev 1.2)
[ ] Model thermal resistance parameters of probe ribbon
[ ] Characterize thermal resistance of probe ribbon
[ ] Convert pin headers on probe ribbon to FFC
[ ] Write requirements for sensor hub: USB port, MCU, 0.1" headers, pull-up resistors, flat flex connectors.
[ ] Part selection for sensor hub
[ ] Sensor hub PCB schematic
[ ] Sensor hub PCB layout
[ ] Fabricate and validate sensor hub PCB

## Firmware
[X] Write minimal TMP117 driver
[X] Complete serial data format specification
[X] Write microcontroller firmware bridging TMP117 to serial (ESP32C6)
[ ] Replace ad-hoc TMP117 driver with [tmp117-rs](https://github.com/xgroleau/tmp117-rs)
[ ] Port firmware to lower-cost USB-serial microcontrollers: RP2040/RP2350/CH32V

## Software
[X] Write userspace driver bridging serial stream to .npyz timeseries with Unix timestamp
[X] Validate userspace driver end-to-end with application
[ ] Generalize userspace driver to remove assumption of three sensors
[ ] Create udev rules to automatically start and stop userspace driver when device is connected
[ ] Validate that udev rules behave as expected
[ ] Make userspace driver persistent to device serial connection interruptions: don't reset timeseries
[ ] Enable [atomic filesystem writes](https://github.com/danburkert/fs2-rs) for serial-to-file userspace driver 
[ ] Convert userspace driver to [kernel space driver](https://not-matthias.github.io/posts/kernel-driver-with-rust-2022/)
[ ] Add macOS and Windows support

