use std::io;

pub use rust_db::Binary;

use crate::helper::flat_remove_errors;

pub trait Binary {
    fn from_bin(data: &[u8], path: &str) -> io::Result<Self>
    where
        Self: Sized;
    fn into_bin(&self, path: &str) -> io::Result<Vec<u8>>;
    fn bin_size() -> usize;
    fn delete(&self, path: &str) -> io::Result<()>;
}

impl Binary for char {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(data[0] as char)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![*self as u8])
    }
    fn bin_size() -> usize {
        1
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for bool {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(data[0] != 0)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![*self as u8])
    }
    fn bin_size() -> usize {
        1
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for u8 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(data[0])
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![*self])
    }
    fn bin_size() -> usize {
        1
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for u16 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(((data[0] as u16) << 8) + (data[1] as u16))
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![(*self >> 8) as u8, *self as u8])
    }
    fn bin_size() -> usize {
        2
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for u32 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(((data[0] as u32) << 24)
            + ((data[1] as u32) << 16)
            + ((data[2] as u32) << 8)
            + (data[3] as u32))
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ])
    }
    fn bin_size() -> usize {
        4
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for u64 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(((data[0] as u64) << 56)
            + ((data[1] as u64) << 48)
            + ((data[2] as u64) << 40)
            + ((data[3] as u64) << 32)
            + ((data[4] as u64) << 24)
            + ((data[5] as u64) << 16)
            + ((data[6] as u64) << 8)
            + (data[7] as u64))
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ])
    }
    fn bin_size() -> usize {
        8
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for u128 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(((data[0] as u128) << 120)
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
            + (data[15] as u128))
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![
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
        ])
    }
    fn bin_size() -> usize {
        16
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for i8 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(data[0] as i8)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![*self as u8])
    }
    fn bin_size() -> usize {
        1
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for i16 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok((((data[0] as u16) << 8) + (data[1] as u16)) as i16)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![(*self >> 8) as u8, *self as u8])
    }
    fn bin_size() -> usize {
        2
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for i32 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok((((data[0] as u32) << 24)
            + ((data[1] as u32) << 16)
            + ((data[2] as u32) << 8)
            + (data[3] as u32)) as i32)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ])
    }
    fn bin_size() -> usize {
        4
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for i64 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok((((data[0] as u64) << 56)
            + ((data[1] as u64) << 48)
            + ((data[2] as u64) << 40)
            + ((data[3] as u64) << 32)
            + ((data[4] as u64) << 24)
            + ((data[5] as u64) << 16)
            + ((data[6] as u64) << 8)
            + (data[7] as u64)) as i64)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ])
    }
    fn bin_size() -> usize {
        8
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for i128 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok((((data[0] as u128) << 120)
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
            + (data[15] as u128)) as i128)
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(vec![
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
        ])
    }
    fn bin_size() -> usize {
        16
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for f32 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(f32::from_bits(
            ((data[0] as u32) << 24)
                + ((data[1] as u32) << 16)
                + ((data[2] as u32) << 8)
                + (data[3] as u32),
        ))
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        let s = self.to_bits();
        Ok(vec![
            (s >> 24) as u8,
            (s >> 16) as u8,
            (s >> 8) as u8,
            s as u8,
        ])
    }
    fn bin_size() -> usize {
        4
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for f64 {
    fn from_bin(data: &[u8], _: &str) -> io::Result<Self> {
        Ok(f64::from_bits(
            ((data[0] as u64) << 56)
                + ((data[1] as u64) << 48)
                + ((data[2] as u64) << 40)
                + ((data[3] as u64) << 32)
                + ((data[4] as u64) << 24)
                + ((data[5] as u64) << 16)
                + ((data[6] as u64) << 8)
                + (data[7] as u64),
        ))
    }
    fn into_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        let s = self.to_bits();
        Ok(vec![
            (s >> 56) as u8,
            (s >> 48) as u8,
            (s >> 40) as u8,
            (s >> 32) as u8,
            (s >> 24) as u8,
            (s >> 16) as u8,
            (s >> 8) as u8,
            s as u8,
        ])
    }
    fn bin_size() -> usize {
        8
    }
    fn delete(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

impl<T, const LEN: usize> Binary for [T; LEN]
where
    T: Binary + Copy + Default,
{
    fn from_bin(data: &[u8], path: &str) -> io::Result<Self> {
        let mut result: [T; LEN] = [T::default(); LEN];
        for (i, item) in result.iter_mut().enumerate() {
            *item = T::from_bin(&data[i * T::bin_size()..], path)?;
        }
        Ok(result)
    }

    fn into_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        flat_remove_errors(self.iter().map(|item| item.into_bin(path)))
    }

    fn bin_size() -> usize {
        T::bin_size() * LEN
    }
    fn delete(&self, path: &str) -> io::Result<()> {
        for item in self {
            item.delete(path)?;
        }
        Ok(())
    }
}
