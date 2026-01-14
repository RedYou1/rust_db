use std::{
    cmp::Ordering,
    io::{self},
    marker::PhantomData,
};

use crate::{
    bd_path::BDPath, bin_file::BaseBinFile, binary::Binary, cached_bin_file::CachedBinFile,
    prelude::BinFile,
};

#[derive(Debug)]
pub enum IndexGet<Row> {
    Found(usize, Vec<Row>),
    NotFound(usize),
    InternalError(String),
    Err(io::Error),
}

impl<Row> From<io::Error> for IndexGet<Row> {
    fn from(value: io::Error) -> Self {
        Self::Err(value)
    }
}

#[derive(Binary, Clone)]
pub struct IndexRow<ColType: Binary + PartialOrd> {
    pub data: ColType,
    pub index: usize,
}

pub trait UnspecifiedIndex<Row: Binary> {
    fn check_unique(&mut self, row: &mut Row) -> io::Result<Option<()>>;
    fn insert(&mut self, index: usize, row: &mut Row) -> io::Result<()>;
    fn remove(&mut self, index: usize) -> std::io::Result<()>;
    fn clear(&mut self) -> io::Result<()>;
    fn clear_cache(&mut self);
}

pub type IndexFile<ColType, Row> = SpecificIndexFile<ColType, Row, BinFile<IndexRow<ColType>>>;
pub type CachedIndexFile<ColType, Row> =
    SpecificIndexFile<ColType, Row, CachedBinFile<IndexRow<ColType>>>;

pub struct SpecificIndexFile<
    ColType: Binary + PartialOrd,
    Row: Binary,
    BinFile: BaseBinFile<IndexRow<ColType>>,
> {
    bin: BinFile,
    index: IdAsIndexFile<ColType, IndexRow<ColType>, BinFile>,
    extract: Box<fn(&Row) -> &ColType>,
    check_unique: bool,
}

impl<ColType: Binary + PartialOrd, Row: Binary, BinFile: BaseBinFile<IndexRow<ColType>>>
    SpecificIndexFile<ColType, Row, BinFile>
{
    pub fn new(
        path: BDPath,
        extract: Box<fn(&Row) -> &ColType>,
        check_unique: bool,
    ) -> io::Result<Self> {
        Ok(Self {
            bin: BinFile::new(path)?,
            index: IdAsIndexFile::new(Box::new(|row: &IndexRow<ColType>, other: &ColType| {
                row.data.partial_cmp(other)
            }))?,
            extract,
            check_unique,
        })
    }

    pub fn indx(&self, find: &ColType) -> IndexGet<IndexRow<ColType>> {
        self.index.indx(&self.bin, find)
    }
}
impl<ColType: Binary + PartialOrd + Clone, Row: Binary, BinFile: BaseBinFile<IndexRow<ColType>>>
    SpecificIndexFile<ColType, Row, BinFile>
{
    fn base_check_unique(&mut self, row: &mut Row) -> io::Result<Option<()>> {
        Ok(if !self.check_unique {
            Some(())
        } else {
            let data = (self.extract)(row).clone();
            if self
                .bin
                .gets(0, None)?
                .iter()
                .any(|row| match row.data.partial_cmp(&data) {
                    None | Some(Ordering::Equal) => true,
                    Some(Ordering::Greater) | Some(Ordering::Less) => false,
                })
            {
                None
            } else {
                Some(())
            }
        })
    }
    fn base_insert(&mut self, index: usize, row: &mut Row) -> io::Result<()> {
        let mut datas = self
            .bin
            .gets(0, None)?
            .into_iter()
            .map(|row| {
                if row.index < index {
                    row
                } else {
                    IndexRow {
                        data: row.data,
                        index: row.index + 1,
                    }
                }
            })
            .collect::<Vec<IndexRow<ColType>>>();
        datas.insert(
            index,
            IndexRow {
                data: (self.extract)(row).clone(),
                index,
            },
        );
        self.bin.clear()?;
        self.bin.inserts(0, &mut datas)?;
        Ok(())
    }

    fn base_remove(&mut self, index: usize) -> std::io::Result<()> {
        let mut datas = self
            .bin
            .gets(0, None)?
            .into_iter()
            .filter_map(|row| {
                if row.index < index {
                    Some(row)
                } else if row.index == index {
                    None
                } else {
                    Some(IndexRow {
                        data: row.data,
                        index: row.index - 1,
                    })
                }
            })
            .collect::<Vec<IndexRow<ColType>>>();
        self.bin.clear()?;
        self.bin.inserts(0, &mut datas)?;
        Ok(())
    }

    fn base_clear(&mut self) -> io::Result<()> {
        self.bin.clear()
    }
}
impl<ColType: Binary + PartialOrd + Clone, Row: Binary> UnspecifiedIndex<Row>
    for SpecificIndexFile<ColType, Row, BinFile<IndexRow<ColType>>>
{
    fn check_unique(&mut self, row: &mut Row) -> io::Result<Option<()>> {
        self.base_check_unique(row)
    }
    fn insert(&mut self, index: usize, row: &mut Row) -> io::Result<()> {
        self.base_insert(index, row)
    }
    fn remove(&mut self, index: usize) -> std::io::Result<()> {
        self.base_remove(index)
    }
    fn clear(&mut self) -> io::Result<()> {
        self.base_clear()
    }
    fn clear_cache(&mut self) {}
}
impl<ColType: Binary + PartialOrd + Clone, Row: Binary> UnspecifiedIndex<Row>
    for SpecificIndexFile<ColType, Row, CachedBinFile<IndexRow<ColType>>>
{
    fn check_unique(&mut self, row: &mut Row) -> io::Result<Option<()>> {
        self.base_check_unique(row)
    }
    fn insert(&mut self, index: usize, row: &mut Row) -> io::Result<()> {
        self.base_insert(index, row)
    }
    fn remove(&mut self, index: usize) -> std::io::Result<()> {
        self.base_remove(index)
    }
    fn clear(&mut self) -> io::Result<()> {
        self.base_clear()
    }
    fn clear_cache(&mut self) {
        self.bin.clear_cache();
    }
}

pub struct IdAsIndexFile<ColType: Binary + PartialOrd, Row: Binary, BinFile: BaseBinFile<Row>> {
    row: PhantomData<(ColType, BinFile)>,
    cmp: Box<fn(&Row, &ColType) -> Option<Ordering>>,
}

impl<ColType: Binary + PartialOrd, Row: Binary, BinFile: BaseBinFile<Row>>
    IdAsIndexFile<ColType, Row, BinFile>
{
    pub fn new(cmp: Box<fn(&Row, &ColType) -> Option<Ordering>>) -> io::Result<Self> {
        Ok(Self {
            row: PhantomData,
            cmp,
        })
    }

    pub fn indx(&self, bin: &BinFile, find: &ColType) -> IndexGet<Row> {
        match bin.len() {
            Ok(0) => IndexGet::NotFound(0),
            Ok(len) => self.bin_search(bin, 0, len - 1, find),
            Err(e) => e.into(),
        }
    }

    fn bin_search(
        &self,
        bin: &BinFile,
        mut from: usize,
        mut to: usize,
        find: &ColType,
    ) -> IndexGet<Row> {
        while from <= to {
            let idx = (to - from) / 2 + from;
            let found = match bin.get(idx) {
                Ok(found) => found,
                Err(e) => return e.into(),
            };
            match (self.cmp)(&found, find) {
                Some(Ordering::Equal) => {
                    let from = match self.expand_min(bin, idx, find) {
                        Ok(found) => found,
                        Err(e) => return e.into(),
                    };
                    let to = match self.expand_max(bin, idx, find) {
                        Ok(found) => found,
                        Err(e) => return e.into(),
                    };

                    return match (from..=to)
                        .map(|i| bin.get(i))
                        .collect::<io::Result<Vec<Row>>>()
                    {
                        Ok(ok) => IndexGet::Found(from, ok),
                        Err(e) => IndexGet::Err(e),
                    };
                }
                Some(Ordering::Greater) if idx == from => return IndexGet::NotFound(idx),
                Some(Ordering::Greater) => to = idx - 1,

                Some(Ordering::Less) if idx == to => return IndexGet::NotFound(idx + 1),
                Some(Ordering::Less) => from = idx + 1,

                None => return IndexGet::InternalError("cmp error".to_owned()),
            }
        }
        IndexGet::InternalError("index bin_search went outside of range".to_owned())
    }

    fn expand_min(&self, bin: &BinFile, mut idx: usize, find: &ColType) -> io::Result<usize> {
        if idx == 0 {
            return Ok(idx);
        }
        idx -= 1;
        while let Some(Ordering::Equal) = (self.cmp)(&bin.get(idx)?, find) {
            if idx == 0 {
                return Ok(idx);
            }
            idx -= 1;
        }
        Ok(idx + 1)
    }

    fn expand_max(&self, bin: &BinFile, mut idx: usize, find: &ColType) -> io::Result<usize> {
        let max = bin.len()?;
        if idx + 1 == max {
            return Ok(idx);
        }
        idx += 1;
        while let Some(Ordering::Equal) = (self.cmp)(&bin.get(idx)?, find) {
            if idx + 1 == max {
                return Ok(idx);
            }
            idx += 1;
        }
        Ok(idx - 1)
    }
}
