pub trait Binary {
    fn from_bin(data: &[u8]) -> Self;
    fn into_bin(&self) -> Vec<u8>;
    fn bin_size() -> usize;
}

impl Binary for char {
    fn from_bin(data: &[u8]) -> Self {
        data[0] as char
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![*self as u8]
    }
    fn bin_size() -> usize {
        1
    }
}

impl Binary for bool {
    fn from_bin(data: &[u8]) -> Self {
        data[0] != 0
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![*self as u8]
    }
    fn bin_size() -> usize {
        1
    }
}

impl Binary for u8 {
    fn from_bin(data: &[u8]) -> Self {
        data[0]
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![*self]
    }
    fn bin_size() -> usize {
        1
    }
}

impl Binary for u16 {
    fn from_bin(data: &[u8]) -> Self {
        ((data[0] as u16) << 8) + (data[1] as u16)
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![(*self >> 8) as u8, *self as u8]
    }
    fn bin_size() -> usize {
        2
    }
}

impl Binary for u32 {
    fn from_bin(data: &[u8]) -> Self {
        ((data[0] as u32) << 24)
            + ((data[1] as u32) << 16)
            + ((data[2] as u32) << 8)
            + (data[3] as u32)
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ]
    }
    fn bin_size() -> usize {
        4
    }
}

impl Binary for u64 {
    fn from_bin(data: &[u8]) -> Self {
        ((data[0] as u64) << 56)
            + ((data[1] as u64) << 48)
            + ((data[2] as u64) << 40)
            + ((data[3] as u64) << 32)
            + ((data[4] as u64) << 24)
            + ((data[5] as u64) << 16)
            + ((data[6] as u64) << 8)
            + (data[7] as u64)
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ]
    }
    fn bin_size() -> usize {
        8
    }
}

impl Binary for u128 {
    fn from_bin(data: &[u8]) -> Self {
        ((data[0] as u128) << 120)
            + ((data[1] as u128) << 112)
            + ((data[2] as u128) << 104)
            + ((data[3] as u128) << 96)
            + ((data[4] as u128) << 88)
            + ((data[5] as u128) << 80)
            + ((data[6] as u128) << 72)
            + ((data[7] as u128) << 64)
            + ((data[8] as u128) << 56)
            + ((data[9] as u128) << 48)
            + ((data[10] as u128) << 40)
            + ((data[11] as u128) << 32)
            + ((data[12] as u128) << 24)
            + ((data[13] as u128) << 16)
            + ((data[14] as u128) << 8)
            + (data[15] as u128)
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![
            (*self >> 120) as u8,
            (*self >> 112) as u8,
            (*self >> 104) as u8,
            (*self >> 96) as u8,
            (*self >> 88) as u8,
            (*self >> 80) as u8,
            (*self >> 72) as u8,
            (*self >> 64) as u8,
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ]
    }
    fn bin_size() -> usize {
        16
    }
}

impl Binary for i8 {
    fn from_bin(data: &[u8]) -> Self {
        data[0] as i8
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![*self as u8]
    }
    fn bin_size() -> usize {
        1
    }
}

impl Binary for i16 {
    fn from_bin(data: &[u8]) -> Self {
        (((data[0] as u16) << 8) + (data[1] as u16)) as i16
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![(*self >> 8) as u8, *self as u8]
    }
    fn bin_size() -> usize {
        2
    }
}

impl Binary for i32 {
    fn from_bin(data: &[u8]) -> Self {
        (((data[0] as u32) << 24)
            + ((data[1] as u32) << 16)
            + ((data[2] as u32) << 8)
            + (data[3] as u32)) as i32
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ]
    }
    fn bin_size() -> usize {
        4
    }
}

impl Binary for i64 {
    fn from_bin(data: &[u8]) -> Self {
        (((data[0] as u64) << 56)
            + ((data[1] as u64) << 48)
            + ((data[2] as u64) << 40)
            + ((data[3] as u64) << 32)
            + ((data[4] as u64) << 24)
            + ((data[5] as u64) << 16)
            + ((data[6] as u64) << 8)
            + (data[7] as u64)) as i64
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ]
    }
    fn bin_size() -> usize {
        8
    }
}

impl Binary for i128 {
    fn from_bin(data: &[u8]) -> Self {
        (((data[0] as u128) << 120)
            + ((data[1] as u128) << 112)
            + ((data[2] as u128) << 104)
            + ((data[3] as u128) << 96)
            + ((data[4] as u128) << 88)
            + ((data[5] as u128) << 80)
            + ((data[6] as u128) << 72)
            + ((data[7] as u128) << 64)
            + ((data[8] as u128) << 56)
            + ((data[9] as u128) << 48)
            + ((data[10] as u128) << 40)
            + ((data[11] as u128) << 32)
            + ((data[12] as u128) << 24)
            + ((data[13] as u128) << 16)
            + ((data[14] as u128) << 8)
            + (data[15] as u128)) as i128
    }
    fn into_bin(self: &Self) -> Vec<u8> {
        vec![
            (*self >> 120) as u8,
            (*self >> 112) as u8,
            (*self >> 104) as u8,
            (*self >> 96) as u8,
            (*self >> 88) as u8,
            (*self >> 80) as u8,
            (*self >> 72) as u8,
            (*self >> 64) as u8,
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ]
    }
    fn bin_size() -> usize {
        16
    }
}
