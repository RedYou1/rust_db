pub use crate::binary::Binary;
use crate::helper::{flat_remove_errors, read_all};
use std::{
    fs::{create_dir_all, File},
    os::windows::fs::FileExt,
};

pub struct Table<'a, Row>
where
    Row: Binary,
{
    path: &'a str,
    datas: Vec<Row>,
}

impl<'a, Row> Table<'a, Row>
where
    Row: Binary,
{
    pub fn new(path: &'a str) -> std::io::Result<Self> {
        let datas = read_all(path);
        if let Err(_) = datas {
            create_dir_all(format!("{path}/dyn"))?;
            File::create(format!("{path}/main.bin").as_str())?;
        }
        Ok(Table {
            path: path,
            datas: datas.unwrap_or_default(),
        })
    }

    pub fn strict_new(path: &'a str) -> std::io::Result<Self> {
        Ok(Table {
            path: path,
            datas: read_all(path)?,
        })
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn get(&self, index: usize) -> Option<&Row> {
        self.datas.get(index)
    }

    pub fn datas(&self) -> &Vec<Row> {
        &self.datas
    }

    pub fn len(&self) -> usize {
        self.datas.len()
    }

    pub fn insert(&mut self, index: usize, data: Row) -> std::io::Result<()> {
        self.datas.insert(index, data);

        let file = File::create(format!("{}/main.bin", self.path).as_str())?;
        let bin = flat_remove_errors(
            self.datas[index..]
                .into_iter()
                .map(|row| row.into_bin(self.path)),
        )?;
        file.seek_write(&bin, (index * Row::bin_size()) as u64)?;
        file.sync_all()
    }

    pub fn remove(&mut self, index: usize) -> std::io::Result<()> {
        let item = self.datas.remove(index);
        item.delete(self.path)?;

        let file = File::create(format!("{}/main.bin", self.path).as_str())?;
        let bin = flat_remove_errors(
            self.datas[index..]
                .into_iter()
                .map(|row| row.into_bin(self.path)),
        )?;
        file.seek_write(&bin, (index * Row::bin_size()) as u64)?;
        file.sync_all()
    }
}
