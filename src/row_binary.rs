use crate::binary::Binary;
use std::{fmt::Display, io, marker::PhantomData};

pub trait RowBinary<ID>
where
    ID: Binary + Display,
{
    fn from_row_bin(data: &[u8], id: &ID, path: &str) -> io::Result<Self>
    where
        Self: Sized;
    fn as_row_bin(&self, id: &ID, path: &str) -> io::Result<Vec<u8>>;
    fn row_bin_size(_: PhantomData<ID>) -> usize;
    fn row_delete(&self, id: &ID, path: &str) -> io::Result<()>;
}

impl<ID: Binary + Display, Bin: Binary> RowBinary<ID> for Bin {
    fn from_row_bin(data: &[u8], _: &ID, path: &str) -> io::Result<Self>
    where
        Self: Sized,
    {
        Bin::from_bin(data, path)
    }
    fn as_row_bin(&self, _: &ID, path: &str) -> io::Result<Vec<u8>> {
        self.as_bin(path)
    }
    fn row_bin_size(_: PhantomData<ID>) -> usize {
        Bin::bin_size()
    }
    fn row_delete(&self, _: &ID, path: &str) -> io::Result<()> {
        self.delete(path)
    }
}
