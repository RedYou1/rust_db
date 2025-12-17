use std::{
    collections::HashMap,
    fs::{File, read_dir, remove_file},
    hash::Hash,
    io::{self, Error, Read, Write},
    num::NonZero,
};

use crate::bin_file::Binary;

#[derive(Debug, Clone)]
pub struct DynanicBinary<DATA>
where
    DATA: AsBinary,
{
    id: Option<NonZero<usize>>,
    data: DATA,
}

impl<DATA: AsBinary + PartialEq> PartialEq for DynanicBinary<DATA> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

pub trait AsBinary {
    fn from_as_bin(data: Vec<u8>, path: &str) -> io::Result<Self>
    where
        Self: Sized;
    fn as_as_bin(&self, path: &str) -> io::Result<Vec<u8>>;
}

impl<DATA> DynanicBinary<DATA>
where
    DATA: AsBinary,
{
    pub const fn new(data: DATA) -> Self {
        DynanicBinary { id: None, data }
    }

    pub const fn id(&self) -> Option<NonZero<usize>> {
        self.id
    }

    pub(crate) fn id_error(&self) -> Result<NonZero<usize>, Error> {
        self.id.ok_or(io::Error::other("id is not set"))
    }

    pub const fn data(&self) -> &DATA {
        &self.data
    }

    pub const fn mut_data(&mut self) -> &mut DATA {
        &mut self.data
    }
}

impl<DATA> Binary for DynanicBinary<DATA>
where
    DATA: AsBinary,
{
    fn from_bin(data: &[u8], path: &str) -> io::Result<Self> {
        let id =
            NonZero::new(usize::from_bin(data, path)?).ok_or(io::Error::other("id is zero"))?;

        let mut file = File::open(format!("{path}/dyn/{id}.bin"))?;
        let mut result = vec![0; file.metadata()?.len() as usize];
        file.read_exact(&mut result)?;
        Ok(DynanicBinary {
            id: Some(id),
            data: DATA::from_as_bin(result, path)?,
        })
    }

    fn as_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        let id = self.id.unwrap_or_else(move || unsafe {
            NonZero::new_unchecked(
                read_dir(format!("{path}/dyn"))
                    .expect("need dyn dir")
                    .count()
                    + 1,
            )
        });
        let mut file = File::create(format!("{path}/dyn/{id}.bin"))?;
        file.write_all(&self.data.as_as_bin(path)?)?;
        file.sync_all()?;

        id.get().as_bin(path)
    }

    fn bin_size() -> usize {
        usize::bin_size()
    }

    fn delete(&self, path: &str) -> io::Result<()> {
        remove_file(format!("{path}/dyn/{}.bin", self.id_error()?))
    }
}

impl AsBinary for String {
    fn from_as_bin(data: Vec<u8>, _: &str) -> io::Result<Self> {
        String::from_utf8(data).map_err(io::Error::other)
    }

    fn as_as_bin(&self, _: &str) -> io::Result<Vec<u8>> {
        Ok(self.bytes().collect())
    }
}

impl<T> AsBinary for Vec<T>
where
    T: Binary,
{
    fn from_as_bin(data: Vec<u8>, path: &str) -> io::Result<Self> {
        data.chunks(T::bin_size())
            .map(|row| T::from_bin(row, path))
            .collect()
    }

    fn as_as_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        Ok(self
            .iter()
            .flat_map(|item| item.as_bin(path))
            .flatten()
            .collect())
    }
}

impl<K, V> AsBinary for HashMap<K, V>
where
    K: Binary + Eq + Hash,
    V: Binary,
{
    fn from_as_bin(data: Vec<u8>, path: &str) -> io::Result<Self>
    where
        Self: Sized,
    {
        Ok(data
            .chunks(K::bin_size() + V::bin_size())
            .flat_map(|data| -> io::Result<(K, V)> {
                Ok((
                    K::from_bin(data, path)?,
                    V::from_bin(&data[K::bin_size()..], path)?,
                ))
            })
            .collect())
    }

    fn as_as_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        Ok(self
            .iter()
            .flat_map(|t| -> io::Result<Vec<u8>> {
                let mut a = t.0.as_bin(path)?;
                a.extend_from_slice(&t.1.as_bin(path)?);
                Ok(a)
            })
            .flatten()
            .collect())
    }
}
