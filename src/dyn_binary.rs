use std::{
    collections::HashMap,
    fmt::Display,
    fs::{remove_file, File},
    hash::Hash,
    io::{self, Read, Write},
};

use crate::bin_file::Binary;

#[derive(Debug, Clone, PartialEq)]
pub struct DynanicBinary<ID, DATA>
where
    ID: Binary + Display,
    DATA: AsBinary,
{
    id: ID,
    data: DATA,
}

pub trait AsBinary {
    fn from_as_bin(data: Vec<u8>, path: &str) -> io::Result<Self>
    where
        Self: Sized;
    fn into_as_bin(&self, path: &str) -> io::Result<Vec<u8>>;
}

impl<ID, DATA> DynanicBinary<ID, DATA>
where
    ID: Binary + Display,
    DATA: AsBinary,
{
    pub fn new(id: ID, data: DATA) -> Self {
        DynanicBinary { id: id, data: data }
    }

    pub fn id(&self) -> &ID {
        &self.id
    }

    pub fn data(&self) -> &DATA {
        &self.data
    }

    pub fn mut_data(&mut self) -> &mut DATA {
        &mut self.data
    }
}

impl<ID, DATA> Binary for DynanicBinary<ID, DATA>
where
    ID: Binary + Display,
    DATA: AsBinary,
{
    fn from_bin(data: &[u8], path: &str) -> io::Result<Self> {
        let id = ID::from_bin(data, path)?;

        let mut file = File::open(format!("{path}/dyn/{id}.bin"))?;
        let mut result = vec![0 as u8; file.metadata()?.len() as usize];
        file.read(&mut result)?;
        Ok(DynanicBinary {
            id: id,
            data: DATA::from_as_bin(result, path)?,
        })
    }

    fn into_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::create(format!("{path}/dyn/{}.bin", self.id))?;
        file.write_all(&self.data.into_as_bin(path)?)?;
        file.sync_all()?;

        self.id.into_bin(path)
    }

    fn bin_size() -> usize {
        ID::bin_size()
    }

    fn delete(&self, path: &str) -> io::Result<()> {
        remove_file(format!("{path}/dyn/{}.bin", self.id))
    }
}

impl AsBinary for String {
    fn from_as_bin(data: Vec<u8>, _: &str) -> io::Result<Self> {
        String::from_utf8(data).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn into_as_bin(&self, _: &str) -> io::Result<Vec<u8>> {
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

    fn into_as_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        Ok(self
            .into_iter()
            .flat_map(|item| item.into_bin(path))
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

    fn into_as_bin(&self, path: &str) -> io::Result<Vec<u8>> {
        Ok(self
            .into_iter()
            .flat_map(|t| -> io::Result<Vec<u8>> {
                let mut a = t.0.into_bin(path)?;
                a.extend_from_slice(&t.1.into_bin(path)?);
                Ok(a)
            })
            .flatten()
            .collect())
    }
}
