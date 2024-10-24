
pub const LITTLE_ENDIAN: u8 = 0x1;
pub const BIG_ENDIAN: u8 = 0x2;

pub const RWX_EXEC: u8 = 0x1;
pub const RWX_WRITE: u8 = 0x2;
pub const RWX_READ: u8 = 0x4;

pub fn read_u16_from_u8_vec(bytes: &Vec<u8>, start: usize, endianness: u8) -> u16 {
    let b: &[u8; 2] = (&bytes[start..start+2]).try_into().unwrap();
    match endianness { 
        LITTLE_ENDIAN => u16::from_le_bytes(*b), 
        BIG_ENDIAN => u16::from_be_bytes(*b),
        _ => panic!("unknown endian type {}", endianness)
    }
}

pub fn read_u32_from_u8_vec(bytes: &Vec<u8>, start: usize, endianness: u8) -> u32 {
    let b: &[u8; 4] = (&bytes[start..start+4]).try_into().unwrap();
    match endianness { 
        LITTLE_ENDIAN => u32::from_le_bytes(*b), 
        BIG_ENDIAN => u32::from_be_bytes(*b),
        _ => panic!("unknown endian type {}", endianness)
    }
}

pub fn read_u64_from_u8_vec(bytes: &Vec<u8>, start: usize, endianness: u8) -> u64 {
    let b: &[u8; 8] = (&bytes[start..start+8]).try_into().unwrap();
    match endianness { 
        LITTLE_ENDIAN => u64::from_le_bytes(*b), 
        BIG_ENDIAN => u64::from_be_bytes(*b),
        _ => panic!("unknown endian type {}", endianness)
    }
}

pub fn read_u32_to_u64_from_u8_vec(bytes: &Vec<u8>, start: usize, endianness: u8, ) -> u64 {
    let b: &[u8; 4] = (&bytes[start..start+4]).try_into().unwrap();
    u64::from(match endianness { 
        LITTLE_ENDIAN => u32::from_le_bytes(*b), 
        BIG_ENDIAN => u32::from_be_bytes(*b),
        _ => panic!("unknown endian type {}", endianness)
    })
}

pub fn i32_sign(x: i32) -> &'static str {
    if x < 0 { "-" } else { "+" }
}

pub trait BitExtr {
    fn bextr(self, start: u32, stop: u32) -> Self;
}

impl BitExtr for u32 {
    fn bextr(self, start: u32, stop: u32) -> Self {
        assert!(start < Self::BITS && stop <= start);
        (self << (Self::BITS - start - 1)) >> (Self::BITS - (start - stop) - 1)
    }
}

impl BitExtr for u16 {
    fn bextr(self, start: u32, stop: u32) -> Self {
        assert!(start < Self::BITS && stop <= start);
        (self << (Self::BITS - start - 1)) >> (Self::BITS - (start - stop) - 1)
    }
}

impl BitExtr for i32 {
    fn bextr(self, start: u32, stop: u32) -> Self {
        assert!(start < Self::BITS && stop <= start);
        (self << (Self::BITS - start - 1)) >> (Self::BITS - (start - stop) - 1)
    }
}

impl BitExtr for i16 {
    fn bextr(self, start: u32, stop: u32) -> Self {
        assert!(start < Self::BITS && stop <= start);
        (self << (Self::BITS - start - 1)) >> (Self::BITS - (start - stop) - 1)
    }
}
