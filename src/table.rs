pub use crate::row_binary::RowBinary;
use crate::{bin_file::BinFile, binary::Binary};
pub use rust_db::TableRow;
use std::{io, marker::PhantomData};

pub trait TableRow<ID>: Binary
where
    ID: Binary + PartialEq,
{
    fn id(&self) -> &ID;
}

pub struct Table<'a, Row, ID>
where
    ID: Binary + PartialEq,
    Row: TableRow<ID>,
{
    bin: BinFile<'a, Row>,
    phantom_id: PhantomData<ID>,
}

impl<'a, Row, ID> Table<'a, Row, ID>
where
    ID: Binary + PartialEq,
    Row: TableRow<ID>,
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
                .find(|(_, row)| *row.id() == *id)
                .ok_or(io::Error::new(io::ErrorKind::Other, "Element not found"))?
                .0,
            Some(1),
        )
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.bin.clear()
    }
}
