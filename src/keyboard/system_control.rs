use atmega_usbd::UsbBus;
use avr_device::atmega32u4::USB_DEVICE;
use usb_device::Result;
use usbd_hid::descriptor::{SerializedDescriptor, SystemControlReport};
use usbd_hid::hid_class::{
    HIDClass, HidClassSettings, HidProtocol, HidSubClass, ProtocolModeConfig,
};

use super::*;

const fn hid_class_settings() -> HidClassSettings {
    HidClassSettings {
        subclass: HidSubClass::NoSubClass,
        protocol: HidProtocol::Keyboard,
        config: ProtocolModeConfig::DefaultBehavior,
        locale: keyboard_locale(),
    }
}

pub struct Keyboard {
    usb_bus: KeyboardUsbBusAllocator,
    report: KeyboardReport,
    last_report: KeyboardReport,
}

impl Keyboard {
    /// Creates a new [Keyboard] device, taking ownership of the `USB_DEVICE` register of the
    /// ATmega32u4
    pub fn new(usb: USB_DEVICE) -> Self {
        Self {
            usb_bus: UsbBus::new(usb),
            report: KeyboardReport::default(),
            last_report: KeyboardReport::default(),
        }
    }
}

impl From<KeyboardUsbBusAllocator> for Keyboard {
    /// Creates a [Keyboard] device from a UsbBusAllocator.
    ///
    /// Useful for converting from other keyboard types. Ensures unique ownership over the
    /// underlying UsbBus.
    fn from(usb_bus: KeyboardUsbBusAllocator) -> Self {
        Self {
            usb_bus,
            report: KeyboardReport::default(),
            last_report: KeyboardReport::default(),
        }
    }
}

impl KeyboardOps for Keyboard {
    fn report(&self) -> &KeyboardReport {
        &self.report
    }

    fn set_report(&mut self, report: KeyboardReport) {
        self.report = report;
    }

    fn report_mut(&mut self) -> &mut KeyboardReport {
        &mut self.report
    }

    fn last_report(&self) -> &KeyboardReport {
        &self.last_report
    }

    fn last_report_mut(&mut self) -> &mut KeyboardReport {
        &mut self.last_report
    }

    fn bus(&self) -> &KeyboardUsbBusAllocator {
        &self.usb_bus
    }

    fn end(&mut self) -> Result<()> {
        self.release_all();
        self.send_report()
    }

    fn send_report(&mut self) -> Result<()> {
        if self.keycodes_changed() {
            let hid_class = HIDClass::new_ep_in_with_settings(
                self.bus(),
                SystemControlReport::desc(),
                POLL_MS,
                hid_class_settings(),
            );

            let report = self.report();
            // replace the Ok(usize) with Ok(())
            let ret = hid_class.push_input(report).map(|_| ());
            self.last_report = self.report;

            ret
        } else {
            Ok(())
        }
    }

    fn press(&mut self, key: u8) -> usize {
        let mut done = false;
        let is_system_control_key = is_system_control(key);

        if is_system_control_key {
            for keycode in self.report.keycodes.iter_mut() {
                if *keycode == key {
                    done = true;
                    break;
                }

                if *keycode == 0 {
                    done = true;
                    *keycode = key;
                    break;
                }
            }
        }

        (done && is_system_control_key) as usize
    }

    fn release(&mut self, key: u8) -> usize {
        if is_system_control(key) {
            // it's some other key:
            // Test the key report to see if the key is present. Clear it if it exists.
            // Check all positions in case the key is present more than once (which it shouldn't be)
            for keycode in self.report.keycodes.iter_mut() {
                if *keycode == key {
                    *keycode = 0;
                }
            }

            utils::sort_keycodes(self.report.keycodes.as_mut());
        }

        1
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        let mut found = false;

        for &keycode in self.report.keycodes.iter() {
            if keycode == key {
                found = true;
                break;
            }
        }

        found && is_system_control(key)
    }

    fn was_key_pressed(&self, key: u8) -> bool {
        let mut found = false;

        for &keycode in self.last_report.keycodes.iter() {
            if keycode == key {
                found = true;
                break;
            }
        }

        found && is_system_control(key)
    }

    fn to_usb_bus(self) -> KeyboardUsbBusAllocator {
        self.usb_bus
    }
}
