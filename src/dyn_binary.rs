use std::{
    fs::{remove_file, File},
    io::{Read, Write},
    path::Path,
};

use crate::table::Binary;

#[derive(Debug, Clone, PartialEq)]
pub struct DynanicBinary<T>
where
    T: AsBinary,
{
    id: u64,
    data: T,
}

pub trait AsBinary {
    fn from_bin(data: Vec<u8>) -> Self;
    fn into_bin(&self) -> Vec<u8>;
}

fn get_path(path: &str) -> u64 {
    for id in 0.. {
        let path = format!("{path}/{id}.bin");
        let path = path.as_str();
        if !Path::new(path).exists() {
            File::create(path).unwrap();
            return id;
        }
    }
    panic!("get_path out of id");
}

impl<T> DynanicBinary<T>
where
    T: AsBinary,
{
    pub fn new(path: &str, data: T) -> Self {
        DynanicBinary {
            id: get_path(path),
            data: data,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn mut_data(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Binary for DynanicBinary<T>
where
    T: AsBinary,
{
    fn from_bin(data: &[u8], path: &str) -> Self {
        let id = u64::from_bin(data, path);

        let mut file = File::open(format!("{path}/{id}.bin")).unwrap();
        let mut result = vec![0 as u8; file.metadata().unwrap().len() as usize];
        file.read(&mut result).unwrap();
        DynanicBinary {
            id: id,
            data: T::from_bin(result),
        }
    }

    fn into_bin(&self, path: &str) -> Vec<u8> {
        let mut file = File::create(format!("{path}/{}.bin", self.id)).unwrap();
        file.write_all(&self.data.into_bin()).unwrap();
        file.sync_all().unwrap();

        self.id.into_bin(path)
    }

    fn bin_size() -> usize {
        u64::bin_size()
    }

    fn delete(&self, path: &str) {
        remove_file(format!("{path}/{}.bin", self.id)).unwrap();
    }
}

impl AsBinary for String {
    fn from_bin(data: Vec<u8>) -> Self {
        String::from_utf8(data).unwrap()
    }

    fn into_bin(&self) -> Vec<u8> {
        self.bytes().collect::<Vec<u8>>()
    }
}
