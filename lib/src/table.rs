use crate::{
    bd_path::BDPath,
    bin_file::BaseBinFile,
    binary::Binary,
    index_file::{IdAsIndexFile, Index, IndexFile, IndexGet, UnspecifiedIndex},
    prelude::{BinFile, CachedBinFile},
};
pub use rust_db_macro::Table;
use std::{
    cmp::Ordering,
    fs::{create_dir, remove_dir_all},
    io::{self, Error},
};

pub trait Table: Binary + Clone
where
    Self::ID: Binary + PartialOrd<Self::ID>,
{
    type ID;

    fn id(&self) -> &Self::ID;
    fn id_cmp(&self, other: &Self::ID) -> Option<Ordering> {
        self.id().partial_cmp(other)
    }
    fn get_indexes(path: String) -> io::Result<Vec<Box<dyn UnspecifiedIndex<Self>>>>;
}

#[derive(Debug)]
pub enum TableGet<Ok> {
    Found(Ok),
    NotFound,
    InternalError(String),
    Err(io::Error),
}

pub type TableFile<Row> = SpecificTableFile<Row, BinFile<Row>>;
pub type CachedTableFile<Row> = SpecificTableFile<Row, CachedBinFile<Row>>;

pub struct SpecificTableFile<Row: Table, RowBinFile: BaseBinFile<Row>> {
    bin: RowBinFile,
    id_index: IdAsIndexFile<Row::ID, Row>,
    other_index: Vec<Box<dyn UnspecifiedIndex<Row>>>,
}

impl<Row: Table, BinFile: BaseBinFile<Row>> SpecificTableFile<Row, BinFile> {
    pub fn new(path: String) -> io::Result<Self> {
        let path = BDPath {
            dir_path: path,
            rel_file_path: "main.bin".to_owned(),
        };
        Ok(Self {
            other_index: Table::get_indexes(path.dir_path.clone())?,
            bin: BinFile::new(path.clone())?,
            id_index: IdAsIndexFile::new(path, Box::new(Row::id_cmp))?,
        })
    }

    pub fn get_by_index(&self, index: usize) -> io::Result<Row> {
        self.bin.get(index)
    }

    pub fn get_by_id(&self, id: &Row::ID) -> TableGet<Row> {
        match &match self.id_index.indx(id) {
            IndexGet::Found(_, index) => index,
            IndexGet::NotFound(_) => return TableGet::NotFound,
            IndexGet::InternalError(e) => return TableGet::InternalError(e),
            IndexGet::Err(e) => return TableGet::Err(e),
        }[..]
        {
            [] => TableGet::InternalError("index returned an empty array".to_owned()),
            [data] => TableGet::Found(data.clone()),
            _ => TableGet::InternalError("multiple with the same id".to_owned()),
        }
    }

    pub fn get_all(&self) -> io::Result<Vec<Row>> {
        self.bin.gets(0, None)
    }

    pub fn is_empty(&self) -> io::Result<bool> {
        self.bin.is_empty()
    }

    pub fn len(&self) -> io::Result<usize> {
        self.bin.len()
    }

    pub fn insert(&mut self, data: &mut Row) -> std::io::Result<bool> {
        for index_file in &mut self.other_index {
            if index_file.check_unique(data)?.is_none() {
                return Ok(false);
            }
        }

        let index = match self.id_index.indx(data.id()) {
            IndexGet::Found(_, _) => return Ok(false),
            IndexGet::NotFound(i) => i,
            IndexGet::InternalError(e) => return Err(Error::other(e)),
            IndexGet::Err(e) => return Err(e),
        };

        self.bin.insert(index, data)?;
        for index_file in &mut self.other_index {
            index_file.insert(index, data)?;
        }
        Ok(true)
    }

    pub fn remove(&mut self, id: &Row::ID) -> std::io::Result<()> {
        let (index, datas) = match self.id_index.indx(id) {
            IndexGet::Found(index, datas) => (index, datas),
            IndexGet::NotFound(_) => return Err(Error::other("remove not found")),
            IndexGet::InternalError(e) => return Err(Error::other(e)),
            IndexGet::Err(e) => return Err(e),
        };
        match &datas[..] {
            [] => Err(Error::other("index returned an empty array")),
            [_] => {
                for index_file in &mut self.other_index {
                    index_file.remove(index)?;
                }
                self.bin.remove(index, Some(1))?;
                Ok(())
            }
            _ => Err(Error::other("multiple with the same id")),
        }
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.bin.clear()?;
        remove_dir_all(self.bin.path().dyn_path())?;
        create_dir(self.bin.path().dyn_path())?;
        for index_file in &mut self.other_index {
            index_file.clear()?;
        }
        Ok(())
    }

    /// # Safety
    /// Don't call it by yourself.
    /// It is used by the Table macro.
    pub unsafe fn get_index_file<ColType: Binary + PartialOrd>(
        &self,
        index: usize,
    ) -> &IndexFile<ColType, Row> {
        unsafe {
            (self.other_index[index].as_ref() as *const dyn UnspecifiedIndex<Row>
                as *const IndexFile<ColType, Row>)
                .as_ref()
                .expect("downcast of index file in table.")
        }
    }
}

impl<Row: Table> CachedTableFile<Row> {
    pub fn remove_from_cache(&mut self, id: &<Row as Table>::ID) -> TableGet<()> {
        let (index, datas) = match self.id_index.indx(id) {
            IndexGet::Found(index, datas) => (index, datas),
            IndexGet::NotFound(_) => return TableGet::InternalError("remove not found".to_owned()),
            IndexGet::InternalError(e) => return TableGet::InternalError(e),
            IndexGet::Err(e) => return TableGet::Err(e),
        };
        match &datas[..] {
            [] => TableGet::InternalError("index returned an empty array".to_owned()),
            [_] => {
                self.bin.remove_from_cache(index, Some(1));
                TableGet::Found(())
            }
            _ => TableGet::InternalError("multiple with the same id".to_owned()),
        }
    }
}

impl<Row: Table, BinFile: BaseBinFile<Row>> AsRef<BinFile> for SpecificTableFile<Row, BinFile> {
    fn as_ref(&self) -> &BinFile {
        &self.bin
    }
}
