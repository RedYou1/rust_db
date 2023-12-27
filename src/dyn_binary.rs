use std::{
    fmt::Display,
    fs::{remove_file, File},
    io::{Read, Write},
};

use crate::table::Binary;

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
    fn from_bin(data: Vec<u8>, path: &str) -> Self;
    fn into_bin(&self, path: &str) -> Vec<u8>;
}

impl<ID, DATA> DynanicBinary<ID, DATA>
where
    ID: Binary + Display,
    DATA: AsBinary,
{
    pub fn new(id: ID, data: DATA) -> Self {
        DynanicBinary { id: id, data: data }
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
    fn from_bin(data: &[u8], path: &str) -> Self {
        let id = ID::from_bin(data, path);

        let mut file = File::open(format!("{path}/{id}.bin")).unwrap();
        let mut result = vec![0 as u8; file.metadata().unwrap().len() as usize];
        file.read(&mut result).unwrap();
        DynanicBinary {
            id: id,
            data: DATA::from_bin(result, path),
        }
    }

    fn into_bin(&self, path: &str) -> Vec<u8> {
        let mut file = File::create(format!("{path}/{}.bin", self.id)).unwrap();
        file.write_all(&self.data.into_bin(path)).unwrap();
        file.sync_all().unwrap();

        self.id.into_bin(path)
    }

    fn bin_size() -> usize {
        ID::bin_size()
    }

    fn delete(&self, path: &str) {
        remove_file(format!("{path}/{}.bin", self.id)).unwrap();
    }
}

impl AsBinary for String {
    fn from_bin(data: Vec<u8>, _: &str) -> Self {
        String::from_utf8(data).unwrap()
    }

    fn into_bin(&self, _: &str) -> Vec<u8> {
        self.bytes().collect::<Vec<u8>>()
    }
}

impl<T> AsBinary for Vec<T>
where
    T: Binary,
{
    fn from_bin(data: Vec<u8>, path: &str) -> Self {
        data.chunks(T::bin_size())
            .map(|row| T::from_bin(row, path))
            .collect::<Vec<T>>()
    }

    fn into_bin(&self, path: &str) -> Vec<u8> {
        self.into_iter()
            .flat_map(|item| item.into_bin(path))
            .collect()
    }
}
