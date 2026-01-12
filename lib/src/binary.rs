use std::io::{self, Error};

pub use rust_db_macro::Binary;

use crate::{bd_path::BDPath, dyn_binary::AsBinary};

pub trait Binary: Sized + AsBinary {
    fn from_bin(data: &[u8], path: &BDPath) -> io::Result<Self>;
    fn as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>>;
    fn bin_size() -> usize;
    fn delete(&self, path: &BDPath) -> io::Result<()>;
}

impl<T: Binary> AsBinary for T {
    fn from_as_bin(data: Vec<u8>, path: &BDPath) -> io::Result<Self> {
        Self::from_bin(&data, path)
    }
    fn as_as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>> {
        self.as_bin(path)
    }
}

impl Binary for char {
    fn from_bin(data: &[u8], _: &BDPath) -> io::Result<Self> {
        Ok(data[0] as char)
    }
    fn as_bin(&mut self, _: &BDPath) -> io::Result<Vec<u8>> {
        Ok(vec![*self as u8])
    }
    fn bin_size() -> usize {
        1
    }
    fn delete(&self, _: &BDPath) -> io::Result<()> {
        Ok(())
    }
}

impl Binary for bool {
    fn from_bin(data: &[u8], _: &BDPath) -> io::Result<Self> {
        Ok(data[0] != 0)
    }
    fn as_bin(&mut self, _: &BDPath) -> io::Result<Vec<u8>> {
        Ok(vec![*self as u8])
    }
    fn bin_size() -> usize {
        1
    }
    fn delete(&self, _: &BDPath) -> io::Result<()> {
        Ok(())
    }
}

macro_rules! to_binary {
    ($($self: ty),+) => {
        to_binary!($($self: $self),+);
    };
    ($($self: ty: $as: ty),+) => {
        $(
            impl Binary for $self {
                fn from_bin(data: &[u8], _: &BDPath) -> io::Result<Self> {
                    Ok(Self::from_le_bytes(data[..Self::bin_size()].try_into().map_err(Error::other)?))
                }
                fn as_bin(&mut self, _: &BDPath) -> io::Result<Vec<u8>> {
                    Ok(Vec::from(self.to_le_bytes()))
                }
                fn bin_size() -> usize {
                    <$as>::BITS as usize / 8
                }
                fn delete(&self, _: &BDPath) -> io::Result<()> {
                    Ok(())
                }
            }
        )+
    };
}
to_binary!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize
);
to_binary!(f32: u32, f64: u64);

impl<T, const LEN: usize> Binary for [T; LEN]
where
    T: Binary + Copy + Default,
{
    fn from_bin(data: &[u8], path: &BDPath) -> io::Result<Self> {
        let mut result: [T; LEN] = [T::default(); LEN];
        for (i, item) in result.iter_mut().enumerate() {
            *item = T::from_bin(&data[i * T::bin_size()..], path)?;
        }
        Ok(result)
    }

    fn as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>> {
        Ok(self
            .iter_mut()
            .flat_map(|item| item.as_bin(path))
            .flatten()
            .collect())
    }

    fn bin_size() -> usize {
        T::bin_size() * LEN
    }
    fn delete(&self, path: &BDPath) -> io::Result<()> {
        for item in self {
            item.delete(path)?;
        }
        Ok(())
    }
}
