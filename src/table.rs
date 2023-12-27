use std::{
    fs::{create_dir_all, read_dir, File},
    io::Read,
    os::windows::fs::FileExt,
};

pub use crate::binary::Binary;

pub struct Table<'a, Row>
where
    Row: Binary,
{
    path: &'a str,
    datas: Vec<Row>,
}

fn read_all<Row>(path: &str) -> std::io::Result<Vec<Row>>
where
    Row: Binary,
{
    let mut file = File::open(format!("{path}/main.bin").as_str())?;
    let mut result = vec![0 as u8; file.metadata()?.len() as usize];
    file.read(&mut result)?;
    Ok(result
        .chunks(Row::bin_size())
        .map(|row| Row::from_bin(row, path))
        .collect::<Vec<Row>>())
}

impl<'a, Row> Table<'a, Row>
where
    Row: Binary,
{
    pub fn new(path: &'a str) -> std::io::Result<Self> {
        let datas = read_all(path);
        if let Err(_) = datas {
            create_dir_all(path)?;
            File::create(format!("{path}/main.bin").as_str())?;
        }
        Ok(Table {
            path: path,
            datas: datas.unwrap_or_default(),
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

    pub fn nb_files(&self) -> usize {
        read_dir(self.path).unwrap().count()
    }

    pub fn insert(&mut self, index: usize, data: Row) -> std::io::Result<()> {
        self.datas.insert(index, data);

        let file = File::create(format!("{}/main.bin", self.path).as_str())?;
        let bin = self.datas[index..]
            .into_iter()
            .flat_map(|row| row.into_bin(self.path))
            .collect::<Vec<u8>>();
        file.seek_write(&bin, (index * Row::bin_size()) as u64)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> std::io::Result<()> {
        let item = self.datas.remove(index);
        item.delete(self.path);

        let file = File::create(format!("{}/main.bin", self.path).as_str())?;
        let bin = self.datas[index..]
            .into_iter()
            .flat_map(|row| row.into_bin(self.path))
            .collect::<Vec<u8>>();
        file.seek_write(&bin, (index * Row::bin_size()) as u64)?;
        file.sync_all()?;
        Ok(())
    }
}
