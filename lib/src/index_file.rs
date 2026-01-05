use std::{
    cmp::Ordering,
    io::{self},
    marker::PhantomData,
};

use crate::{bd_path::BDPath, bin_file::BinFile, binary::Binary};

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

#[derive(Binary)]
pub struct IndexRow<ColType: Binary + PartialOrd> {
    pub data: ColType,
    pub index: usize,
}

pub trait UnspecifiedIndex<Row: Binary> {
    fn check_unique(&mut self, row: &mut Row) -> io::Result<Option<()>>;
    fn insert(&mut self, index: usize, row: &mut Row) -> io::Result<()>;
    fn remove(&mut self, index: usize) -> std::io::Result<()>;
    fn clear(&mut self) -> io::Result<()>;
}
pub trait Index<ColType: Binary + PartialOrd, Row: Binary> {
    fn indx(&self, find: &ColType) -> IndexGet<Row>;
}

pub struct IndexFile<ColType: Binary + PartialOrd, Row: Binary> {
    index: IdAsIndexFile<ColType, IndexRow<ColType>>,
    extract: Box<fn(&Row) -> &ColType>,
    check_unique: bool,
}

impl<ColType: Binary + PartialOrd, Row: Binary> IndexFile<ColType, Row> {
    pub fn new(
        path: BDPath,
        extract: Box<fn(&Row) -> &ColType>,
        check_unique: bool,
    ) -> io::Result<Self> {
        Ok(Self {
            index: IdAsIndexFile::new(
                path,
                Box::new(|row: &IndexRow<ColType>, other: &ColType| row.data.partial_cmp(other)),
            )?,
            extract,
            check_unique,
        })
    }
}
impl<ColType: Binary + PartialOrd, Row: Binary> Index<ColType, IndexRow<ColType>>
    for IndexFile<ColType, Row>
{
    fn indx(&self, find: &ColType) -> IndexGet<IndexRow<ColType>> {
        self.index.indx(find)
    }
}
impl<ColType: Binary + PartialOrd + Clone, Row: Binary> UnspecifiedIndex<Row>
    for IndexFile<ColType, Row>
{
    fn check_unique(&mut self, row: &mut Row) -> io::Result<Option<()>> {
        Ok(if !self.check_unique {
            Some(())
        } else {
            let data = (self.extract)(row).clone();
            if self
                .index
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
    fn insert(&mut self, index: usize, row: &mut Row) -> io::Result<()> {
        let mut datas = self
            .index
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
        self.index.bin.clear()?;
        self.index.bin.inserts(0, datas.into_iter())?;
        Ok(())
    }

    fn remove(&mut self, index: usize) -> std::io::Result<()> {
        let datas = self
            .index
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
        self.index.bin.clear()?;
        self.index.bin.inserts(0, datas.into_iter())?;
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.index.bin.clear()
    }
}

pub struct IdAsIndexFile<ColType: Binary + PartialOrd, Row: Binary> {
    bin: BinFile<Row>,
    row: PhantomData<ColType>,
    cmp: Box<fn(&Row, &ColType) -> Option<Ordering>>,
}

impl<ColType: Binary + PartialOrd, Row: Binary> IdAsIndexFile<ColType, Row> {
    pub fn new(path: BDPath, cmp: Box<fn(&Row, &ColType) -> Option<Ordering>>) -> io::Result<Self> {
        Ok(Self {
            bin: BinFile::new(path)?,
            row: PhantomData,
            cmp,
        })
    }

    fn bin_search(&self, mut from: usize, mut to: usize, find: &ColType) -> IndexGet<Row> {
        while from <= to {
            let idx = (to - from) / 2 + from;
            let found = match self.bin.get(idx) {
                Ok(found) => found,
                Err(e) => return e.into(),
            };
            match (self.cmp)(&found, find) {
                Some(Ordering::Equal) => {
                    let from = match self.expand_min(idx, find) {
                        Ok(found) => found,
                        Err(e) => return e.into(),
                    };
                    let to = match self.expand_max(idx, find) {
                        Ok(found) => found,
                        Err(e) => return e.into(),
                    };

                    return match (from..=to)
                        .map(|i| self.bin.get(i))
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

    fn expand_min(&self, mut idx: usize, find: &ColType) -> io::Result<usize> {
        if idx == 0 {
            return Ok(idx);
        }
        idx -= 1;
        while let Some(Ordering::Equal) = (self.cmp)(&self.bin.get(idx)?, find) {
            if idx == 0 {
                return Ok(idx);
            }
            idx -= 1;
        }
        Ok(idx + 1)
    }

    fn expand_max(&self, mut idx: usize, find: &ColType) -> io::Result<usize> {
        let max = self.bin.len()?;
        if idx + 1 == max {
            return Ok(idx);
        }
        idx += 1;
        while let Some(Ordering::Equal) = (self.cmp)(&self.bin.get(idx)?, find) {
            if idx + 1 == max {
                return Ok(idx);
            }
            idx += 1;
        }
        Ok(idx - 1)
    }
}
impl<ColType: Binary + PartialOrd, Row: Binary> Index<ColType, Row>
    for IdAsIndexFile<ColType, Row>
{
    fn indx(&self, find: &ColType) -> IndexGet<Row> {
        match self.bin.len() {
            Ok(0) => IndexGet::NotFound(0),
            Ok(len) => self.bin_search(0, len - 1, find),
            Err(e) => e.into(),
        }
    }
}
