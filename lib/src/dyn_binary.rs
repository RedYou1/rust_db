use std::{
    collections::HashMap,
    fs::{File, create_dir_all, remove_file},
    hash::Hash,
    io::{self, Error, Read, Write},
    num::NonZero,
    path::Path,
};

use crate::{bd_path::BDPath, binary::Binary};

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

impl<DATA: AsBinary + PartialOrd> PartialOrd for DynanicBinary<DATA> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

pub trait AsBinary: Sized {
    fn from_as_bin(data: Vec<u8>, path: &BDPath) -> io::Result<Self>;
    fn as_as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>>;
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
    fn from_bin(data: &[u8], path: &BDPath) -> io::Result<Self> {
        let id = NonZero::new(usize::from_bin(data, path)?)
            .ok_or_else(|| io::Error::other("id is zero"))?;

        let mut file = File::open(format!("{}/{id}.bin", path.dyn_path()))?;
        let mut result = vec![0; file.metadata()?.len() as usize];
        file.read_exact(&mut result)?;
        Ok(DynanicBinary {
            id: Some(id),
            data: DATA::from_as_bin(result, path)?,
        })
    }

    fn as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>> {
        let id = self.id.get_or_insert_with(move || unsafe {
            NonZero::new_unchecked({
                if !Path::new(path.dyn_path().as_str()).exists() {
                    create_dir_all(path.dyn_path()).expect("creation of dyn folder");
                }
                loop {
                    let r = match usize::BITS {
                        8 => rand::random::<u8>() as usize,
                        16 => rand::random::<u16>() as usize,
                        32 => rand::random::<u32>() as usize,
                        64 => rand::random::<u64>() as usize,
                        128 => rand::random::<u128>() as usize,
                        _ => panic!("usize size not defined"),
                    };
                    if r == 0 {
                        continue;
                    }

                    if !Path::new(BDPath::new_dyn(path.dir_path.clone(), r).full().as_str())
                        .exists()
                    {
                        break r;
                    }
                }
            })
        });
        let mut file = File::create(format!("{}/{id}.bin", path.dyn_path()))?;
        file.write_all(&self.data.as_as_bin(path)?)?;
        file.sync_all()?;

        id.get().as_bin(path)
    }

    fn bin_size() -> usize {
        usize::bin_size()
    }

    fn delete(&self, path: &BDPath) -> io::Result<()> {
        remove_file(format!("{}/{}.bin", path.dyn_path(), self.id_error()?))
    }
}

impl AsBinary for String {
    fn from_as_bin(data: Vec<u8>, _: &BDPath) -> io::Result<Self> {
        String::from_utf8(data).map_err(io::Error::other)
    }

    fn as_as_bin(&mut self, _: &BDPath) -> io::Result<Vec<u8>> {
        Ok(self.bytes().collect())
    }
}

impl<T> AsBinary for Vec<T>
where
    T: Binary,
{
    fn from_as_bin(data: Vec<u8>, path: &BDPath) -> io::Result<Self> {
        data.chunks(T::bin_size())
            .map(|row| T::from_bin(row, path))
            .collect()
    }

    fn as_as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>> {
        Ok(self
            .iter_mut()
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
    fn from_as_bin(data: Vec<u8>, path: &BDPath) -> io::Result<Self>
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

    fn as_as_bin(&mut self, path: &BDPath) -> io::Result<Vec<u8>> {
        Ok(self
            .iter_mut()
            .flat_map(|t| -> io::Result<Vec<u8>> {
                let mut a = unsafe { (t.0 as *const K as *mut K).as_mut() }
                    .ok_or(io::Error::other("HashMap as_bin. key cant mutate"))?
                    .as_bin(path)?;
                a.extend_from_slice(&t.1.as_bin(path)?);
                Ok(a)
            })
            .flatten()
            .collect())
    }
}
