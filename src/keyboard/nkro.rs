use atmega_usbd::UsbBus;
use avr_device::atmega32u4::USB_DEVICE;
use usb_device::{class_prelude::UsbBusAllocator, Result};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use usbd_hid::hid_class::{HIDClass, HidClassSettings, HidProtocol, HidSubClass, ProtocolModeConfig};

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
    usb_bus: UsbBusAllocator<UsbBus<()>>,
    report: KeyboardReport,
    last_report: KeyboardReport,
}

impl Keyboard {
    /// Creates a new [Keyboard] device, taking ownership of the `USB_DEVICE` register of the
    /// ATmega32u4.
    pub fn new(usb: USB_DEVICE) -> Self {
        Self {
            usb_bus: UsbBus::new(usb),
            report: KeyboardReport::default(),
            last_report: KeyboardReport::default(),
        }
    }

    fn send_report_unchecked(&self) -> Result<usize> {
        let hid_class = HIDClass::new_ep_in_with_settings(
            self.bus(),
            KeyboardReport::desc(),
            POLL_MS,
            hid_class_settings(),
        );

        hid_class.push_input(&self.last_report)
    }

}

impl KeyboardOps for Keyboard {
    type UsbBus = UsbBusAllocator<UsbBus<()>>;

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

    fn bus(&self) -> &Self::UsbBus {
        &self.usb_bus
    }

    fn end(&mut self) -> Result<()> {
        self.release_all();
        self.send_report_unchecked()?;
        Ok(())
    }

    fn press(&mut self, key: u8) -> usize {
        if is_printable(key) {
            // If the key is in the range of printable keys
            self.report.keycodes[key_to_index(key)] |= key_to_printable_bitfield(key);
            1
        } else if is_modifier(key) {
            // It's a modifier key, convert key into bitfield
            self.report.modifier |= key_to_modifier_bitfield(key);
            1
        } else {
            0
        }
    }

    fn release(&mut self, key: u8) -> usize {
        if is_printable(key) {
            // If we're releasing a printable key
            self.report.keycodes[key_to_index(key)] &= !key_to_printable_bitfield(key);
            1
        } else if is_modifier(key) {
            // It's a modifier key
            self.report.modifier &= !key_to_modifier_bitfield(key);
            1
        } else {
            // No empty/pressed key was found
            0
        }
    }

    fn send_report(&mut self) -> Result<()> {
        let old_modifiers = self.last_report.modifier;
        let new_modifiers = self.report.modifier;

        let changed_modifiers = old_modifiers ^ new_modifiers;

        if changed_modifiers != 0 {
            // There was at least one modifier change (toggled on or off), remove any
            // non-modifiers from the stored previous report that toggled off in the new
            // report, and send it to the host.
            let mut non_modifiers_toggled_off = false;

            for (last_key, key) in self.last_report.keycodes.iter_mut().zip(self.report.keycodes.iter()) {
                let released_keycodes = *last_key & !key;
                if released_keycodes != 0 {
                    *last_key &= !released_keycodes;
                    non_modifiers_toggled_off = true;
                }
            }

            if non_modifiers_toggled_off {
                self.send_report_unchecked()?;
            }

            self.last_report.modifier = new_modifiers;
            self.send_report_unchecked()?;
        }

        if self.keycodes_changed() {
            self.last_report.keycodes.copy_from_slice(self.report.keycodes.as_ref());
            self.send_report_unchecked()?;
        }

        Ok(())
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        is_printable(key) && self.report.keycodes[key_to_index(key)] & key_to_printable_bitfield(key) != 0
    }

    fn was_key_pressed(&self, key: u8) -> bool {
        is_printable(key) && self.last_report.keycodes[key_to_index(key)] & key_to_printable_bitfield(key) != 0
    }
}
