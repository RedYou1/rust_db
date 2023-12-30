pub use crate::binary::Binary;
use crate::helper::{flat_remove_errors, remove_errors};
use std::{
    fs::{self, create_dir_all, File},
    io::{self},
    marker::PhantomData,
    os::windows::fs::FileExt,
    path::Path,
};

pub struct Table<'a, Row>
where
    Row: Binary + Clone,
{
    path: &'a str,
    phantom: PhantomData<Row>,
}

impl<'a, Row> Table<'a, Row>
where
    Row: Binary + Clone,
{
    pub fn new(path: &'a str) -> std::io::Result<Self> {
        if !Path::new(path).exists() {
            create_dir_all(format!("{path}/dyn"))?;
            File::create(format!("{}/main.bin", path).as_str())?;
        }
        Ok(Table {
            path: path,
            phantom: PhantomData::default(),
        })
    }

    pub fn strict_new(path: &'a str) -> Self {
        Table {
            path: path,
            phantom: PhantomData::default(),
        }
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn get(&self, index: usize) -> io::Result<Option<Row>> {
        Ok(self
            .gets(index, Some(1))?
            .first()
            .map(|value| value.clone()))
    }

    pub fn gets(&self, index: usize, len: Option<usize>) -> io::Result<Vec<Row>> {
        let first_byte = index * Row::bin_size();
        let file_len = self.file_len()?;

        let len = match len {
            Some(len) => {
                let len = len * Row::bin_size();
                if (first_byte + len) > file_len {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("first_byte:{} < file_len:{}", first_byte + len, file_len),
                    ));
                }
                len
            }
            None => {
                if first_byte > file_len {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("first_byte:{} < file_len:{}", first_byte, file_len),
                    ));
                }
                file_len - first_byte
            }
        };
        let mut result = vec![0 as u8; len];
        
        let file = File::open(format!("{}/main.bin", self.path))?;
        file.seek_read(&mut result, first_byte as u64)?;
        remove_errors(
            result
                .chunks(Row::bin_size())
                .map(|row| Row::from_bin(row, self.path)),
        )
    }

    fn file_len(&self) -> io::Result<usize> {
        Ok(fs::metadata(format!("{}/main.bin", self.path))?.len() as usize)
    }

    pub fn len(&self) -> io::Result<usize> {
        Ok(fs::metadata(format!("{}/main.bin", self.path))?.len() as usize / Row::bin_size())
    }

    pub fn insert(&mut self, index: usize, data: Row) -> std::io::Result<()> {
        let mut datas = self.gets(index, None)?;
        datas.insert(0, data);
        let bin = flat_remove_errors(datas.into_iter().map(|row| row.into_bin(self.path)))?;

        let file = File::create(format!("{}/main.bin", self.path).as_str())?;
        file.seek_write(&bin, (index * Row::bin_size()) as u64)?;
        file.sync_all()
    }

    pub fn remove(&mut self, index: usize) -> std::io::Result<()> {
        let file_len = self.file_len()?;
        let mut datas = self.gets(index, None)?;
        datas.remove(0).delete(self.path)?;
        let bin = if (index + 1) > file_len {
            vec![]
        } else {
            flat_remove_errors(datas.into_iter().map(|row| row.into_bin(self.path)))?
        };

        let file = File::create(format!("{}/main.bin", self.path).as_str())?;
        file.seek_write(&bin, (index * Row::bin_size()) as u64)?;
        file.sync_all()
    }
}
