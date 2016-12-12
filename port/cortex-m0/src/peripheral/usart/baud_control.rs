
#[derive(Copy, Clone)]
pub struct USARTBR {
    br: BR,
}

impl USARTBR {
    pub fn new(base_addr: usize) -> Self {
        USARTBR { br: BR::new(base_addr) }
    }

    pub fn set_baud_rate(&self, /* arg?? */) {
        // Need to set baud rate...
    }
}

struct BR {
    base_addr: usize,
}

impl Register for BR {
    fn new(base_addr: usize) -> Self {
        BR { base_addr: base_addr }
    }

    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x0C
    }
}

