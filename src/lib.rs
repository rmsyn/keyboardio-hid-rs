#![no_std]
#![feature(abi_avr_interrupt)]
#![cfg_attr(target_arch = "avr", feature(asm_experimental_arch))]

mod hid_report_observer;
mod hid_settings;
mod keyboard;

pub use hid_report_observer::*;
pub use hid_settings::*;
pub use keyboard::*;

/// Re-export of the [usb-device](https://docs.rs/usb-device/latest/usb_device/) library.
pub use usb_device;
/// Re-export of the [usbd-hid](https://docs.rs/usbd-hid/latest/usbd_hid) library.
pub use usbd_hid;
