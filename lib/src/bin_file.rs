pub use crate::binary::Binary;
use std::{
    fs::{self, File, create_dir_all, remove_dir_all},
    io::{self, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    path::Path,
};

pub struct BinFile<'a, Row>
where
    Row: Binary,
{
    path: &'a str,
    phantom_row: PhantomData<Row>,
}

impl<'a, Row> BinFile<'a, Row>
where
    Row: Binary,
{
    pub fn new(path: &'a str) -> std::io::Result<Self> {
        if !Path::new(path).exists() {
            create_dir_all(format!("{path}/dyn"))?;
            File::create(format!("{path}/main.bin").as_str())?;
        }
        Ok(BinFile {
            path,
            phantom_row: PhantomData,
        })
    }

    pub fn new_default(path: &'a str, default: impl Iterator<Item = Row>) -> std::io::Result<Self> {
        let mut table = BinFile {
            path,
            phantom_row: PhantomData,
        };
        if !Path::new(path).exists() {
            create_dir_all(format!("{path}/dyn"))?;
            File::create(format!("{path}/main.bin").as_str())?;
            table.inserts(0, default)?;
        }
        Ok(table)
    }

    pub const unsafe fn strict_new(path: &'a str) -> Self {
        BinFile {
            path,
            phantom_row: PhantomData,
        }
    }

    pub const fn path(&self) -> &'a str {
        self.path
    }

    pub fn get(&self, index: usize) -> io::Result<Row> {
        let first_byte = index * Row::bin_size();
        let file_len = self.file_len()?;

        if (first_byte + Row::bin_size()) > file_len {
            return Err(io::Error::other(format!(
                "first_byte:{} < file_len:{}",
                first_byte + Row::bin_size(),
                file_len
            )));
        }
        let mut result = vec![0; Row::bin_size()];
        {
            let mut file = File::open(format!("{}/main.bin", self.path))?;
            file.seek(SeekFrom::Start(first_byte as u64))?;
            file.read_exact(&mut result)?;
        }
        Row::from_bin(&result, self.path)
    }

    pub fn gets(&self, index: usize, len: Option<usize>) -> io::Result<Vec<Row>> {
        let first_byte = index * Row::bin_size();
        let file_len = self.file_len()?;

        let len = if let Some(len) = len {
            let len = len * Row::bin_size();
            if (first_byte + len) > file_len {
                return Err(io::Error::other(format!(
                    "first_byte:{} < file_len:{}",
                    first_byte + len,
                    file_len
                )));
            }
            len
        } else {
            if first_byte > file_len {
                return Err(io::Error::other(format!(
                    "first_byte:{first_byte} < file_len:{file_len}"
                )));
            }
            let len = file_len - first_byte;
            if !len.is_multiple_of(Row::bin_size()) {
                return Err(io::Error::other(format!(
                    "len:{} not divisable by {}",
                    len,
                    Row::bin_size()
                )));
            }
            len
        };

        let mut result = vec![0; len];
        {
            let mut file = File::open(format!("{}/main.bin", self.path))?;
            file.seek(SeekFrom::Start(first_byte as u64))?;
            file.read_exact(&mut result)?;
        }

        result
            .chunks(Row::bin_size())
            .map(|row| Row::from_bin(row, self.path))
            .collect()
    }

    fn file_len(&self) -> io::Result<usize> {
        Ok(fs::metadata(format!("{}/main.bin", self.path))?.len() as usize)
    }

    pub fn len(&self) -> io::Result<usize> {
        Ok(fs::metadata(format!("{}/main.bin", self.path))?.len() as usize / Row::bin_size())
    }

    pub fn insert(&mut self, index: usize, data: Row) -> std::io::Result<()> {
        let mut all_datas = self.gets(index, None)?;
        all_datas.insert(0, data);
        let bin: Vec<u8> = all_datas
            .into_iter()
            .flat_map(|row| row.as_bin(self.path))
            .flatten()
            .collect();

        let mut file = File::create(format!("{}/main.bin", self.path).as_str())?;
        file.seek(SeekFrom::Start((index * Row::bin_size()) as u64))?;
        file.write_all(&bin)?;
        file.sync_all()
    }

    pub fn inserts(
        &mut self,
        index: usize,
        datas: impl Iterator<Item = Row>,
    ) -> std::io::Result<()> {
        let mut all_datas = self.gets(index, None)?;
        for (i, data) in datas.enumerate() {
            all_datas.insert(i, data);
        }
        let bin: Vec<u8> = all_datas
            .into_iter()
            .flat_map(|row| row.as_bin(self.path))
            .flatten()
            .collect();

        let mut file = File::create(format!("{}/main.bin", self.path).as_str())?;
        file.seek(SeekFrom::Start((index * Row::bin_size()) as u64))?;
        file.write_all(&bin)?;
        file.sync_all()
    }

    pub fn remove(&mut self, index: usize, len: Option<usize>) -> std::io::Result<()> {
        let mut datas = self.gets(0, None)?;
        for _ in 0..len.unwrap_or(datas.len() - index) {
            datas.remove(index).delete(self.path)?;
        }
        let bin: Vec<u8> = datas
            .into_iter()
            .flat_map(|row| row.as_bin(self.path))
            .flatten()
            .collect();

        let mut file = File::create(format!("{}/main.bin", self.path).as_str())?;
        file.write_all(&bin)?;
        file.sync_all()
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        remove_dir_all(self.path)?;
        create_dir_all(format!("{}/dyn", self.path))?;
        File::create(format!("{}/main.bin", self.path).as_str())?;
        Ok(())
    }
}
