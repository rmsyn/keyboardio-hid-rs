pub struct HStderr {
    buf: [u8; 1024],
}

impl HStderr {
    pub const fn new() -> Self {
        Self { buf: [0u8; 1024] }
    }
}

impl core::fmt::Write for HStderr {
    fn write_str(&mut self, err: &str) -> core::fmt::Result {
        let len = core::cmp::min(err.len(), self.buf.len());
        self.buf.copy_from_slice(err.as_bytes()[..len].as_ref());
        Ok(())
    }
}
