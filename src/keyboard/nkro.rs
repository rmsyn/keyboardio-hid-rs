use usb_device::Result;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use usbd_hid::hid_class::{
    HIDClass, HidClassSettings, HidProtocol, HidSubClass, ProtocolModeConfig,
};

use super::*;

const fn nkro_hid_class_settings() -> HidClassSettings {
    HidClassSettings {
        subclass: HidSubClass::NoSubClass,
        protocol: HidProtocol::Keyboard,
        config: ProtocolModeConfig::DefaultBehavior,
        locale: keyboard_locale(),
    }
}

pub trait NKROKeyboard {
    /// End the keyboard reports.
    fn end(&mut self) -> Result<()>;

    /// Press a key, and add it to the current report.
    ///
    /// Returns 1 if the key is in the printable keycodes, or is a modifier key.
    /// Returns 0 otherwise.
    fn press(&mut self, key: u8) -> usize;

    /// Release a pressed key if the keycode is present in the current report.
    ///
    /// Returns 1 if the key is in the printable keycodes, or is a modifier key.
    /// Returns 0 otherwise.
    fn release(&mut self, key: u8) -> usize;

    /// Sending the current HID report to the host:
    ///
    /// Depending on the differences between the current and previous HID reports, we
    /// might need to send one or two extra reports to guarantee that the host will
    /// process the changes in the correct order. There are two important scenarios
    /// to consider:
    ///
    /// 1. If a non-modifier keycode toggles off in the same report as a modifier
    /// changes, the host might process the modifier change first. For example, if
    /// both `shift` and `4` toggle off in the same report (most likely from a
    /// `LSHIFT(Key_4)` key being released), and that key has been held long enough
    /// to trigger character repeat, we could end up with a plain `4` in the output
    /// at the end of the repeat: `$$$$4` instead of `$$$$$`.
    ///
    /// 2. If a non-modifier keycode toggles on in the same report as a modifier
    /// changes, the host might process the non-modifer first. For example, pressing
    /// and holding an `LSHIFT(Key_4)` key might result in `4$$$` rather than `$$$$`.
    ///
    /// Therefore, each call to `sendReport()` must send (up to) three reports to the
    /// host to guarantee the correct order of processing:
    ///
    /// 1. A report with toggled-off non-modifiers removed.
    /// 2. A report with changes to modifiers.
    /// 3. A report with toggled-on non-modifiers added.
    fn send_report(&mut self) -> Result<()>;

    /// Sends a keyboard report without check report validity.
    fn send_report_unchecked(&self) -> Result<usize>;

    fn hid_class(&self) -> HIDClass<'_, KeyboardUsbBus>;

    /// Gets whether the provided key is pressed in the current keyboard report.
    fn is_key_pressed(&self, key: u8) -> bool;

    /// Gets whether the provided key was pressed in the previous keyboard report.
    fn was_key_pressed(&self, key: u8) -> bool;
}

impl NKROKeyboard for Keyboard {
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

            for (last_key, key) in self
                .last_report
                .keycodes
                .iter_mut()
                .zip(self.report.keycodes.iter())
            {
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
            self.last_report
                .keycodes
                .copy_from_slice(self.report.keycodes.as_ref());
            self.send_report_unchecked()?;
        }

        Ok(())
    }

    fn send_report_unchecked(&self) -> Result<usize> {
        self.hid_class().push_input(&self.last_report)
    }

    fn hid_class(&self) -> HIDClass<'_, KeyboardUsbBus> {
        HIDClass::new_with_settings(
            self.bus(),
            KeyboardReport::desc(),
            POLL_MS,
            nkro_hid_class_settings(),
        )
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        is_printable(key)
            && self.report.keycodes[key_to_index(key)] & key_to_printable_bitfield(key) != 0
    }

    fn was_key_pressed(&self, key: u8) -> bool {
        is_printable(key)
            && self.last_report.keycodes[key_to_index(key)] & key_to_printable_bitfield(key) != 0
    }
}
