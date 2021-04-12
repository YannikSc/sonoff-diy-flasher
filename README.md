# Sonoff DIY Flasher

Flashes your Sonoff devices via the DIY API with a given (ESP) firmware

## What it does

This uses the sonoff DIY api to flash a compatible firmware. This can be the ESP Tasmota firmware, ESPHome or any other compatible firmware.

## Prerequisites

Your Sonoff has to be already connected to your WiFi. This will not require you to setup eWeLink before!

## Usage

Be sure to not flash a broken firmware or interrupt the install process. This software comes without any warrenty. I will not replace your damaged devices!

```bash
sonoff-diy-flasher [PATH_TO_FIRMWARE] [IP_OF_YOUR_SONOFF_DEVICE]

# In the project directory

cargo (+nightly) run -- [PATH_TO_FIRMWARE] [IP_OF_YOUR_SONOFF_DEVICE]

```

## Install

Note: Requires rust **NIGHTLY**

```bash
cargo install sonoff-diy-flasher

# or if nightly is not your default

cargo +nightly install sonoff-diy-flasher
```

## Planed features

- [x] A working CLI utility to flash ESP firmware via the Sonoff DIY API
  - [ ] A progress bar
- [ ] A GUI for the more graphical folk
