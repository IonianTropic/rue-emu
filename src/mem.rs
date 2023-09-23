const MEM_LEN: usize = 0x1000;

pub struct Mem {
    raw: [u8; MEM_LEN],
}

impl Mem {

    pub fn from_buf(buf: Vec<u8>) -> Self {
        let mut raw = [0; MEM_LEN];
        raw[..buf.len()].copy_from_slice(&buf);
        Self {
            raw,
        }
    }

    pub const fn len(&self) -> usize {
        MEM_LEN
    }

    pub fn load_byte(&self, addr: usize) -> u8 {
        let idx = addr % MEM_LEN;
        self.raw[idx]
    }

    pub fn load_half(&self, addr: usize) -> u16 {
        let idx = addr % MEM_LEN;
        let mut bytes = [0u8; 2];
        for i in 0..2 {
            bytes[i] = self.raw[(idx + i) % MEM_LEN];
        }
        u16::from_le_bytes(bytes)
    }

    pub fn load_word(&self, addr: usize) -> u32 {
        let idx = addr % MEM_LEN;
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = self.raw[(idx + i) % MEM_LEN];
        }
        u32::from_le_bytes(bytes)
    }

    pub fn store_byte(&mut self, addr: usize, value: u8) {
        let idx = addr % MEM_LEN;
        self.raw[idx] = value;
    }

    pub fn store_half(&mut self, addr: usize, value: u16) {
        let idx = addr % MEM_LEN;
        let bytes = value.to_le_bytes();
        self.raw[idx..idx + 2].copy_from_slice(&bytes);
    }

    pub fn store_word(&mut self, addr: usize, value: u32) {
        let idx = addr % MEM_LEN;
        let bytes = value.to_le_bytes();
        self.raw[idx..idx + 4].copy_from_slice(&bytes);
    }
}
