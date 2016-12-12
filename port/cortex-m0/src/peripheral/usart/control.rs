// Daniel Seitz and RJ Russell

use super::super::Register;

/// Word length can be 7, 8, or 9 bits.
#[derive(Copy, Clone)]
pub enum WordLength {
    Seven,
    Eight,
    Nine,
}

/// Three USART control registers.
#[derive(Copy, Clone)]
pub struct USARTCR {
    cr1: CR1,
    cr2: CR2,
    cr3: CR3,
}

impl USARTCR {
    pub fn new(base_addr: usize) -> Self {
        USARTCR {
            cr1: CR1::new(base_addr),
            cr2: CR2::new(base_addr),
            cr3: CR3::new(base_addr),
        }
    }

    pub fn set_word_length(&self, length: WordLength) {
        self.cr1.set_word_length(length);
    }
}

#[derive(Copy, Clone)]
struct CR1 {
    base_addr: usize,
}

impl Register for CR1 {
    fn new(base_addr: usize) -> Self {
        CR1 { base_addr: base_addr }
    }

    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x0
    }
}

impl CR1 {
    /*
    fn switch(&self, flip: bool) {
        const UE: usize = 0;

        let mask = match flip {
            false => 0,
            true => 1,
        };

        unsafe {
            let mut reg = self.addr();
            *reg &= !(UE);
            *reg |= mask;
        }
    }
    */
    fn set_word_length(&self, length: WordLength) {
        const M0: usize = 1 << 12;
        const M1: usize = 1 << 28;

        let mask = match length {
            WordLength::Seven => M1,
            WordLength::Eight => 0,
            WordLength::Nine => M0,
        };

        unsafe {
            let mut reg = self.addr();
            *reg &= !(M0 | M1);
            *reg |= mask;
        }
    }
}

#[derive(Copy, Clone)]
struct CR2 {
    base_addr: usize,
}

impl Register for CR2 {
    fn new(base_addr: usize) -> Self {
        CR2 { base_addr: base_addr }
    }

    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x4
    }
}

#[derive(Copy, Clone)]
struct CR3 { base_addr: usize,
}

impl Register for CR3 {
    fn new(base_addr: usize) -> Self {
        CR3 { base_addr: base_addr }
    }

    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x8
    }
}

