use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport, SystemControlReport};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HIDReportId {
    #[default]
    None = 0,
    Mouse,
    Keyboard,
    RawHID,
    ConsumerControl,
    SystemControl,
    Gamepad,
    MouseAbsolute,
    NKROKeyboard,
}

pub enum HIDReport {
    Keyboard(KeyboardReport),
    MediaKeyboardReport(MediaKeyboardReport),
    MouseReport(MouseReport),
    SystemControl(SystemControlReport),
}
