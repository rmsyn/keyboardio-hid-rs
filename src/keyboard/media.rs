use usb_device::Result;
use usbd_hid::hid_class::{
    HidClassSettings, HidProtocol, HidSubClass, ProtocolModeConfig,
};

use super::*;

pub const fn media_hid_class_settings() -> HidClassSettings {
    HidClassSettings {
        subclass: HidSubClass::NoSubClass,
        protocol: HidProtocol::Keyboard,
        config: ProtocolModeConfig::DefaultBehavior,
        locale: keyboard_locale(),
    }
}

pub trait MediaKeyboard {
    /// End the keyboard reports.
    fn end(&mut self) -> Result<()>;

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

    /// Gets whether the provided key is pressed in the current keyboard report.
    fn is_key_pressed(&self, key: u8) -> bool;

    /// Gets whether the provided key was pressed in the previous keyboard report.
    fn was_key_pressed(&self, key: u8) -> bool;
}

impl MediaKeyboard for Keyboard<'_> {
    fn end(&mut self) -> Result<()> {
        self.release_all();
        self.send_report()
    }

    fn send_report(&mut self) -> Result<()> {
        if self.keycodes_changed() {
            let report = self.report().clone();
            // replace the Ok(usize) with Ok(())
            let ret = self.hid_class_mut().push_input(&report).map(|_| ());
            self.last_report = self.report;

            ret
        } else {
            Ok(())
        }
    }

    fn press(&mut self, key: u8) -> usize {
        let mut done = false;
        let is_media_key = is_media(key);

        if is_media_key {
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

        (done && is_media_key) as usize
    }

    fn release(&mut self, key: u8) -> usize {
        if is_media(key) {
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

        found && is_media(key)
    }

    fn was_key_pressed(&self, key: u8) -> bool {
        let mut found = false;

        for &keycode in self.last_report.keycodes.iter() {
            if keycode == key {
                found = true;
                break;
            }
        }

        found && is_media(key)
    }
}
