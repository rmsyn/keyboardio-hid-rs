use atmega_usbd::UsbBus;
use avr_device::atmega32u4::USB_DEVICE;
use usb_device::{Result, class_prelude::UsbBusAllocator};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use usbd_hid::hid_class::{HIDClass, HidClassSettings, HidProtocol, HidSubClass, ProtocolModeConfig};

use crate::hid_settings::{HIDReportId, HIDReport};
use crate::hid_report_observer::HIDReportObserver;

use super::*;

const fn hid_class_settings(protocol: HidProtocol) -> HidClassSettings {
    HidClassSettings {
        subclass: HidSubClass::Boot,
        protocol: protocol,
        config: ProtocolModeConfig::ForceBoot,
        locale: keyboard_locale(),
    }
}

pub struct Keyboard {
    usb_bus: UsbBusAllocator<UsbBus<()>>,
    report: KeyboardReport,
    last_report: KeyboardReport,
    observer: HIDReportObserver,
    default_protocol: HidProtocol,
    protocol: HidProtocol,
    idle: u8,
}

impl Keyboard {
    /// Creates a new [Keyboard] device, taking ownership of the `USB_DEVICE` register of the
    /// ATmega32u4
    pub fn new(usb: USB_DEVICE) -> Self {
        Self {
            usb_bus: UsbBus::new(usb),
            report: KeyboardReport::default(),
            last_report: KeyboardReport::default(),
            observer: HIDReportObserver::default(),
            default_protocol: HidProtocol::Keyboard,
            protocol: HidProtocol::Keyboard,
            idle: 0,
        }
    }

    /// Creates a new [Keyboard] device, taking ownership of the `USB_DEVICE` register of the
    /// ATmega32u4.
    ///
    /// Allows setting a custom [HIDReportObserver] implementation for firing a callback function
    /// on HID report events.
    pub fn new_with_observer(usb: USB_DEVICE, observer: HIDReportObserver) -> Self {
        Self {
            usb_bus: UsbBus::new(usb),
            report: KeyboardReport::default(),
            last_report: KeyboardReport::default(),
            observer,
            default_protocol: HidProtocol::Keyboard,
            protocol: HidProtocol::Keyboard,
            idle: 0,
        }
    }

    /// Gets the currently set protocol for the boot keyboard.
    pub fn protocol(&self) -> HidProtocol {
        self.protocol
    }

    /// Sets the protocol for the boot keyboard.
    pub fn set_protocol(&mut self, protocol: HidProtocol) {
        self.protocol = protocol;
    }

    /// Gets the default protocol for the boot keyboard.
    pub fn default_protocol(&self) -> HidProtocol {
        self.protocol
    }

    pub fn on_usb_reset(&mut self) {
        self.protocol = self.default_protocol;
    }

    /// Gets the idle state of the boot keyboard.
    pub fn idle(&self) -> u8 {
        self.idle
    }
}

impl KeyboardOps for Keyboard {
    fn report(&self) -> &KeyboardReport {
        &self.report
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

    fn end(&mut self) -> Result<()> {
        self.release_all();
        self.send_report()
    }

    fn send_report(&mut self) -> Result<()> {
        if self.keycodes_changed() {
            let hid_class = HIDClass::new_ep_in_with_settings(
                &self.usb_bus,
                KeyboardReport::desc(),
                POLL_MS,
                hid_class_settings(self.protocol),
            );

            let report = self.report();
            // replace the Ok(usize) with Ok(())
            let ret = hid_class.push_input(report).map(|_| ());
            self.observer.observe_report(HIDReportId::Keyboard, HIDReport::Keyboard(*report), &ret);
            self.last_report = self.report;

            ret
        } else {
            Ok(())
        }
    }

    fn press(&mut self, key: u8) -> usize {
        if is_modifier(key) {
            self.report.modifier |= key_to_modifier_bitfield(key);
            1
        } else {
            let mut done = false;

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

            done as usize
        }
    }

    fn release(&mut self, key: u8) -> usize {
        if is_modifier(key) {
            self.report.modifier &= !key_to_modifier_bitfield(key);
        } else {
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

        found && is_printable(key)
    }

    fn was_key_pressed(&self, key: u8) -> bool {
        let mut found = false;

        for &keycode in self.last_report.keycodes.iter() {
            if keycode == key {
                found = true;
                break;
            }
        }

        found && is_printable(key)
    }
}
