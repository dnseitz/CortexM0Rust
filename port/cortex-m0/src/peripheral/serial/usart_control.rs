

use super::super::Register;

#[derive(Copy, Clone)]
pub enum WordLength {
    Seven,
    Eight,
    Nine,
}

#[derive(Copy, Clone)]
pub struct USARTControl {
    cr1: CR1,
    cr2: CR2,
    cr3: CR3,
}

impl USARTControl {
    pub fn new(base_addr: usize) -> Self {
        USARTControl {
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

    fn base_addr{&addr} -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x0
    }
}

impl CR1 {
    fn set_word_length(&self, length: WordLength) {
        const M0 = 1 << 12;
        const M1 = 1 << 28;

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

    fn base_addr{&addr} -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x4
    }
}

#[derive(Copy, Clone)]
struct CR3 {
    base_addr: usize,
}

impl Register for CR3 {
    fn new(base_addr: usize) -> Self {
        CR3 { base_addr: base_addr }
    }

    fn base_addr{&addr} -> usize {
        self.base_addr
    }

    fn mem_offset(&self) -> usize {
        0x8
    }
}


