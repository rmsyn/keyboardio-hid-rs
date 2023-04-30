use usb_device::Result;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage, MediaKey, SystemControlKey};
use usbd_hid::hid_class::HidCountryCode;

pub mod boot;
pub mod nkro;
pub mod media;
pub mod system_control;

pub(crate) const ZERO_KEYS: [u8; 6] = [0u8; 6];
// Polling interval for the host to check USB device reports.
// Higher interval results in better power usage, but slower response time.
// Lower interval results in faster response times, and more power consumption.
//
// FIXME: allow for user-configurable value
#[cfg(feature = "high-performance")]
pub(crate) static POLL_MS: u8 = 10;
#[cfg(feature = "balanced")]
pub(crate) static POLL_MS: u8 = 128;
#[cfg(feature = "best-effort")]
pub(crate) static POLL_MS: u8 = 255;

pub(crate) const fn is_printable(key: u8) -> bool {
    key <= KeyboardUsage::KeypadHexadecimal as u8
}

pub(crate) const fn is_modifier(key: u8) -> bool {
    key >= KeyboardUsage::KeyboardLeftControl as u8 && key <= KeyboardUsage::KeyboardRightGUI as u8
}

pub(crate) fn is_media(key: u8) -> bool {
    MediaKey::from(key) != MediaKey::Reserved
}

pub(crate) fn is_system_control(key: u8) -> bool {
    SystemControlKey::from(key) != SystemControlKey::Reserved
}

pub(crate) const fn key_to_index(key: u8) -> usize {
    (key / 8) as usize
}

pub(crate) const fn key_to_printable_bitfield(key: u8) -> u8 {
    1 << (key % 8)
}

pub(crate) const fn key_to_modifier_bitfield(key: u8) -> u8 {
   1 << (key - KeyboardUsage::KeyboardLeftControl as u8)
}

// FIXME: allow setting locale at runtime by setting config value in device memory.
pub(crate) const fn keyboard_locale() -> HidCountryCode {
    if cfg!(feature = "arabic") {
        HidCountryCode::Arabic
    } else if cfg!(feature = "belgian") {
        HidCountryCode::Belgian
    } else if cfg!(feature = "canadian-bilingual") {
        HidCountryCode::CanadianBilingual
    } else if cfg!(feature = "canadian-french") {
        HidCountryCode::CanadianFrench
    } else if cfg!(feature = "czech") {
        HidCountryCode::CzechRepublic
    } else if cfg!(feature = "danish") {
        HidCountryCode::Danish
    } else if cfg!(feature = "finnish") {
        HidCountryCode::Finnish
    } else if cfg!(feature = "french") {
        HidCountryCode::French
    } else if cfg!(feature = "german") {
        HidCountryCode::German
    } else if cfg!(feature = "greek") {
        HidCountryCode::Greek
    } else if cfg!(feature = "hebrew") {
        HidCountryCode::Hebrew
    } else if cfg!(feature = "hungary") {
        HidCountryCode::Hungary
    } else if cfg!(feature = "international") {
        HidCountryCode::InternationalISO
    } else if cfg!(feature = "italian") {
        HidCountryCode::Italian
    } else if cfg!(feature = "japanese") {
        HidCountryCode::JapanKatakana
    } else if cfg!(feature = "korean") {
        HidCountryCode::Korean
    } else if cfg!(feature = "latin-america") {
        HidCountryCode::LatinAmerica
    } else if cfg!(feature = "netherlands") {
        HidCountryCode::NetherlandsDutch
    } else if cfg!(feature = "norwegian") {
        HidCountryCode::Norwegian
    } else if cfg!(feature = "farsi") {
        HidCountryCode::PersianFarsi
    } else if cfg!(feature = "poland") {
        HidCountryCode::Poland
    } else if cfg!(feature = "portuguese") {
        HidCountryCode::Portuguese
    } else if cfg!(feature = "russia") {
        HidCountryCode::Russia
    } else if cfg!(feature = "slovakia") {
        HidCountryCode::Slovakia
    } else if cfg!(feature = "spanish") {
        HidCountryCode::Spanish
    } else if cfg!(feature = "swedish") {
        HidCountryCode::Swedish
    } else if cfg!(feature = "swiss-french") {
        HidCountryCode::SwissFrench
    } else if cfg!(feature = "swiss-german") {
        HidCountryCode::SwissGerman
    } else if cfg!(feature = "switzerland") {
        HidCountryCode::Switzerland
    } else if cfg!(feature = "taiwan") {
        HidCountryCode::Taiwan
    } else if cfg!(feature = "turkish-q") {
        HidCountryCode::TurkishQ
    } else if cfg!(feature = "uk") {
        HidCountryCode::UK
    } else if cfg!(feature = "us") {
        HidCountryCode::US
    } else if cfg!(feature = "yugoslavia") {
        HidCountryCode::Yugoslavia
    } else if cfg!(feature = "turkish-f") {
        HidCountryCode::TurkishF
    } else {
        HidCountryCode::NotSupported
    }
}

pub trait KeyboardOps {
    type UsbBus;

    /// Gets a reference to the current keyboard report.
    fn report(&self) -> &KeyboardReport;

    /// Gets a mutable reference to the current keyboard report.
    fn report_mut(&mut self) -> &mut KeyboardReport;

    /// Gets a reference to the last keyboard report.
    fn last_report(&self) -> &KeyboardReport;

    /// Gets a mutable reference to the last keyboard report.
    fn last_report_mut(&mut self) -> &mut KeyboardReport;

    /// Gets a reference to the USB bus allocator.
    fn bus(&self) -> &Self::UsbBus;

    /// Begin the keyboard reports (no-op by default).
    fn begin(&self) {}

    /// End the keyboard reports.
    fn end(&mut self) -> Result<()>;

    /// Perform USB device setup.
    fn setup(&mut self) {}

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

    /// Release all keycodes registered in the current keyboard report.
    fn release_all(&mut self) {
        let report = self.report_mut();

        report.modifier = 0;
        report.keycodes.copy_from_slice(ZERO_KEYS.as_ref());
    }

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

    /// Gets whether the keycodes have changed between the last and current keyboard report.
    fn keycodes_changed(&self) -> bool {
        let mut changed = 0;
        for (last, current) in self.last_report().keycodes.iter().zip(self.report().keycodes.iter()) {
            changed |= last ^ current;
        }
        changed != 0
    }

    /// Gets whether the provided key is pressed in the current keyboard report.
    fn is_key_pressed(&self, key: u8) -> bool;

    /// Gets whether the provided key was pressed in the previous keyboard report.
    fn was_key_pressed(&self, key: u8) -> bool;

    /// Returns true if the modifer key passed in will be sent during this key report
    /// Returns false in all other cases
    fn is_modifier_active(&self, key: u8) -> bool {
        is_modifier(key) && self.report().modifier & key_to_modifier_bitfield(key) != 0
    }

    /// Returns true if the modifer key passed in was being sent during the previous key report
    /// Returns false in all other cases
    fn was_modifier_active(&self, key: u8) -> bool {
        is_modifier(key) && self.last_report().modifier & key_to_modifier_bitfield(key) != 0
    }

    /// Returns true if *any* modifier will be sent during this key report
    /// Returns false in all other cases
    fn is_any_modifier_active(&self) -> bool {
        self.report().modifier > 0
    }

    /// Returns true if *any* modifier was being sent during the previous key report
    /// Returns false in all other cases
    fn was_any_modifier_active(&self) -> bool {
        self.last_report().modifier > 0
    }

    /// Gets the number of LEDs in the current keyboard report.
    fn leds(&self) -> u8 {
        self.report().leds
    }

}
