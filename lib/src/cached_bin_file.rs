use std::{cell::RefCell, io};

use crate::{
    A,
    bd_path::BDPath,
    bin_file::{BaseBinFile, BinFile},
    binary::Binary,
    cache::Cache,
};

pub struct CachedBinFile<Row: Binary + Clone> {
    bin: BinFile<Row>,
    cache: RefCell<Cache<Row>>,
}

impl<Row: Binary + Clone> CachedBinFile<Row> {
    pub const fn path(&self) -> &BDPath {
        self.bin.path()
    }

    pub fn cache_len(&self) -> usize {
        self.cache.borrow().len()
    }

    pub fn remove_from_cache(&mut self, index: usize, len: Option<usize>) {
        let mut cache = self.cache.borrow_mut();
        if let Some(len) = len {
            if len > isize::MAX as usize {
                cache.clear();
            } else {
                cache.remove(index, Some(len));
                unsafe {
                    cache.move_cache(index, -(len as isize));
                }
            }
        } else {
            cache.remove(index, None);
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.borrow_mut().clear();
    }
}

impl<Row: Binary + Clone> BaseBinFile<Row> for CachedBinFile<Row> {
    fn new(path: BDPath) -> io::Result<Self> {
        Ok(Self {
            bin: BinFile::new(path)?,
            cache: RefCell::new(Cache::new()),
        })
    }

    fn path(&self) -> &BDPath {
        self.bin.path()
    }

    fn get(&self, index: usize) -> io::Result<Row> {
        let res = {
            let _ = ();
            self.cache.borrow_mut().get(index)
        };
        match res {
            Some(row) => Ok(row),
            None => {
                let data = self.bin.get(index)?;
                self.cache.borrow_mut().insert(index, data.clone());
                Ok(data)
            }
        }
    }

    fn gets(&self, index: usize, len: Option<usize>) -> io::Result<Vec<Row>> {
        let range = index..=(if let Some(len) = len {
            index + len - 1
        } else {
            usize::MAX
        });
        Ok({ self.cache.borrow().chunks() }
            .into_iter()
            .filter_map(|c| {
                let in_cache = c.0;
                c.1.overlap(&range).map(|c| {
                    Ok(if in_cache {
                        self.cache
                            .borrow()
                            .gets(*c.start(), c.len())
                            .expect("chunks return that theirs is data")
                    } else {
                        let data = self.bin.gets(*c.start(), c.len())?;
                        if !data.is_empty() {
                            self.cache
                                .borrow_mut()
                                .inserts(*c.start(), data.iter().cloned());
                        }
                        data
                    })
                })
            })
            .collect::<io::Result<Vec<Vec<Row>>>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    fn is_empty(&self) -> io::Result<bool> {
        self.bin.is_empty()
    }

    fn len(&self) -> io::Result<usize> {
        self.bin.len()
    }

    fn insert(&mut self, index: usize, data: &mut Row) -> std::io::Result<()> {
        unsafe {
            self.cache.borrow_mut().move_cache(index, 1);
        }
        self.bin.insert(index, data)
    }

    fn inserts(&mut self, index: usize, datas: &mut [Row]) -> std::io::Result<()> {
        let mut cache = self.cache.borrow_mut();
        unsafe {
            cache.move_cache(index, datas.len() as isize);
        }
        self.bin.inserts(index, datas)
    }

    fn remove(&mut self, index: usize, len: Option<usize>) -> std::io::Result<()> {
        self.remove_from_cache(index, len);
        self.bin.remove(index, len)
    }

    fn clear(&mut self) -> std::io::Result<()> {
        self.cache.borrow_mut().clear();
        self.bin.clear()
    }
}
