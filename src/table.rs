use crate::{bin_file::BinFile, binary::Binary};
pub use rust_db::IsRow;
use std::{io, marker::PhantomData};

pub trait IsRow<ID>: Binary
where
    ID: Binary + PartialEq,
{
    fn id(&self) -> &ID;
}

pub struct Table<'a, Row, ID>
where
    Row: IsRow<ID>,
    ID: Binary + PartialEq,
{
    bin: BinFile<'a, Row>,
    phantom_id: PhantomData<ID>,
}

impl<'a, Row, ID> Table<'a, Row, ID>
where
    Row: IsRow<ID>,
    ID: Binary + PartialEq,
{
    pub fn new(path: &'a str) -> io::Result<Self> {
        Ok(Table {
            bin: BinFile::new(path)?,
            phantom_id: PhantomData::default(),
        })
    }

    pub fn new_default(path: &'a str, datas: impl Iterator<Item = Row>) -> std::io::Result<Self> {
        Ok(Table {
            bin: BinFile::new_default(path, datas)?,
            phantom_id: PhantomData::default(),
        })
    }

    pub fn strict_new(path: &'a str) -> Self {
        Table {
            bin: BinFile::strict_new(path),
            phantom_id: PhantomData::default(),
        }
    }

    pub fn get(&self, id: &ID) -> io::Result<Row> {
        self.bin
            .gets(0, None)?
            .into_iter()
            .find(|row| *row.id() == *id)
            .ok_or(io::Error::new(io::ErrorKind::Other, "Element not found"))
    }

    pub fn get_all(&self) -> io::Result<Vec<Row>> {
        self.bin.gets(0, None)
    }

    pub fn len(&self) -> io::Result<usize> {
        self.bin.len()
    }

    pub fn insert(&mut self, data: Row) -> std::io::Result<()> {
        self.bin.insert(self.len()?, data)
    }

    pub fn inserts(&mut self, datas: impl Iterator<Item = Row>) -> std::io::Result<()> {
        self.bin.inserts(self.len()?, datas)
    }

    pub fn remove(&mut self, id: &ID) -> std::io::Result<()> {
        self.bin.remove(
            self.bin
                .gets(0, None)?
                .into_iter()
                .enumerate()
                .find(|(index, row)| *row.id() == *id)
                .ok_or(io::Error::new(io::ErrorKind::Other, "Element not found"))?
                .0,
            Some(1),
        )
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.bin.clear()
    }
}
