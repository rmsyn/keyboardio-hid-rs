use arduino_hal::pac::USB_DEVICE;
use atmega_usbd::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage, MediaKey, SystemControlKey};
use usbd_hid::hid_class::{HidCountryCode, HidProtocol};

use crate::HIDReportObserver;

pub mod boot;
pub mod media;
pub mod nkro;
pub mod system_control;

pub type Keycodes = [u8; 6];

pub type KeyboardUsbBus = UsbBus<()>;
pub type KeyboardUsbBusAllocator = UsbBusAllocator<KeyboardUsbBus>;

pub(crate) const ZERO_KEYS: Keycodes = [0u8; 6];
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

pub const fn is_printable(key: u8) -> bool {
    key <= KeyboardUsage::KeypadHexadecimal as u8
}

pub const fn is_modifier(key: u8) -> bool {
    key >= KeyboardUsage::KeyboardLeftControl as u8 && key <= KeyboardUsage::KeyboardRightGUI as u8
}

pub fn is_media(key: u8) -> bool {
    MediaKey::from(key) != MediaKey::Reserved
}

pub fn is_system_control(key: u8) -> bool {
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

pub struct Keyboard {
    usb_bus: KeyboardUsbBusAllocator,
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

    /// Gets a reference to the current keyboard report.
    pub fn report(&self) -> &KeyboardReport {
        &self.report
    }

    /// Sets the current keyboard report.
    pub fn set_report(&mut self, report: KeyboardReport) {
        self.report = report;
    }

    /// Gets a mutable reference to the current keyboard report.
    pub fn report_mut(&mut self) -> &mut KeyboardReport {
        &mut self.report
    }

    /// Gets a reference to the last keyboard report.
    pub fn last_report(&self) -> &KeyboardReport {
        &self.report
    }

    /// Gets a mutable reference to the last keyboard report.
    pub fn last_report_mut(&mut self) -> &mut KeyboardReport {
        &mut self.last_report
    }

    /// Gets a reference to the USB bus allocator.
    pub fn bus(&self) -> &KeyboardUsbBusAllocator {
        &self.usb_bus
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

    /// Switch back to default protocol after a USB reset event.
    pub fn on_usb_reset(&mut self) {
        self.protocol = self.default_protocol;
    }

    /// Gets the idle state of the boot keyboard.
    pub fn idle(&self) -> u8 {
        self.idle
    }

    /// Begin the keyboard reports (no-op by default).
    pub fn begin(&self) {}

    /// Perform USB device setup.
    pub fn setup(&mut self) {}

    /// Release all keycodes registered in the current keyboard report.
    pub fn release_all(&mut self) {
        let report = self.report_mut();

        report.modifier = 0;
        report.keycodes.copy_from_slice(ZERO_KEYS.as_ref());
    }

    /// Gets whether the keycodes have changed between the last and current keyboard report.
    pub fn keycodes_changed(&self) -> bool {
        let mut changed = 0;
        for (last, current) in self
            .last_report()
            .keycodes
            .iter()
            .zip(self.report().keycodes.iter())
        {
            changed |= last ^ current;
        }
        changed != 0
    }

    /// Returns true if the modifer key passed in will be sent during this key report
    /// Returns false in all other cases
    pub fn is_modifier_active(&self, key: u8) -> bool {
        is_modifier(key) && self.report.modifier & key_to_modifier_bitfield(key) != 0
    }

    /// Returns true if the modifer key passed in was being sent during the previous key report
    /// Returns false in all other cases
    pub fn was_modifier_active(&self, key: u8) -> bool {
        is_modifier(key) && self.last_report.modifier & key_to_modifier_bitfield(key) != 0
    }

    /// Returns true if *any* modifier will be sent during this key report
    /// Returns false in all other cases
    pub fn is_any_modifier_active(&self) -> bool {
        self.report.modifier > 0
    }

    /// Returns true if *any* modifier was being sent during the previous key report
    /// Returns false in all other cases
    pub fn was_any_modifier_active(&self) -> bool {
        self.last_report.modifier > 0
    }

    /// Gets the number of LEDs in the current keyboard report.
    pub fn leds(&self) -> u8 {
        self.report.leds
    }

    /// Consumes the [KeyboardOps] implementation object, and returns the underlying USB bus.
    pub fn to_usb_bus(self) -> KeyboardUsbBusAllocator {
        self.usb_bus
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
            observer: HIDReportObserver::default(),
            default_protocol: HidProtocol::Keyboard,
            protocol: HidProtocol::Keyboard,
            idle: 0,
        }
    }
}
