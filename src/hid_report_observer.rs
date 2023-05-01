use usb_device::Result;

use crate::hid_settings::{HIDReport, HIDReportId};

/// Callback function for sending HID reports.
pub type SendReportHook = fn(id: HIDReportId, report: HIDReport, result: &Result<()>);

pub struct HIDReportObserver {
    send_report_hook: Option<SendReportHook>,
}

impl HIDReportObserver {
    #[allow(non_upper_case_globals)]
    const NopSendReportHook: SendReportHook =
        |_id: HIDReportId, _report: HIDReport, _result: &Result<()>| {};

    /// Creates a new [HIDReportObserver].
    pub const fn new(send_report_hook: SendReportHook) -> Self {
        Self {
            send_report_hook: Some(send_report_hook),
        }
    }

    /// Creates a default [HIDReportObserver] with no-op [SendReportHook].
    pub const fn default() -> Self {
        Self {
            send_report_hook: Some(Self::NopSendReportHook),
        }
    }

    /// Attempts to send an HID report by calling the currently set [SendReportHook].
    pub fn observe_report(&self, id: HIDReportId, report: HIDReport, result: &Result<()>) {
        if let Some(send_report_hook) = self.send_report_hook {
            send_report_hook(id, report, result);
        }
    }

    /// Gets the currently set [SendReportHook].
    pub fn hook(&self) -> Option<SendReportHook> {
        self.send_report_hook
    }

    /// Sets the [SendReportHook].
    pub fn set_hook(&mut self, new_hook: SendReportHook) {
        self.send_report_hook = Some(new_hook);
    }
}
