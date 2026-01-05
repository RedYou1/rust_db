use crate::{bd_path::BDPath, binary::Binary};
use std::{
    fs::{self, File, create_dir_all, remove_dir_all, remove_file},
    io::{self, Error, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    path::Path,
};

pub struct BinFile<Row>
where
    Row: Binary,
{
    path: BDPath,
    phantom_row: PhantomData<Row>,
}

impl<Row> BinFile<Row>
where
    Row: Binary,
{
    pub fn new(path: BDPath) -> std::io::Result<Self> {
        if !Path::new(path.full().as_str()).exists() {
            create_dir_all(path.folder())?;
            create_dir_all(path.dyn_path())?;
            File::create(path.full())?;
        } else if !Path::new(path.dyn_path().as_str()).exists() {
            create_dir_all(path.dyn_path())?;
        }
        Ok(BinFile {
            path,
            phantom_row: PhantomData,
        })
    }

    pub const fn path(&self) -> &BDPath {
        &self.path
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
            let mut file = File::open(self.path.full())?;
            file.seek(SeekFrom::Start(first_byte as u64))?;
            file.read_exact(&mut result)?;
        }
        Row::from_bin(&result, &self.path)
    }

    fn read(&self, index: usize, len: Option<usize>) -> io::Result<Vec<u8>> {
        let first_byte = index * Row::bin_size();
        let file_len = self.file_len()?;

        if first_byte == file_len {
            return Ok(Vec::new());
        }
        if first_byte > file_len {
            return Err(Error::other("out of bound"));
        }

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

        let mut file = File::open(self.path.full())?;
        file.seek(SeekFrom::Start(first_byte as u64))?;
        file.read_exact(&mut result)?;

        Ok(result)
    }

    pub fn gets(&self, index: usize, len: Option<usize>) -> io::Result<Vec<Row>> {
        self.read(index, len)?
            .chunks(Row::bin_size())
            .map(|row| Row::from_bin(row, &self.path))
            .collect()
    }

    fn file_len(&self) -> io::Result<usize> {
        Ok(fs::metadata(self.path.full())?.len() as usize)
    }

    pub fn is_empty(&self) -> io::Result<bool> {
        Ok(fs::metadata(self.path.full())?.len() == 0)
    }

    pub fn len(&self) -> io::Result<usize> {
        Ok(fs::metadata(self.path.full())?.len() as usize / Row::bin_size())
    }

    pub fn insert(&mut self, index: usize, data: &mut Row) -> std::io::Result<()> {
        let mut bin = self.read(0, None)?;

        bin.splice(
            (index * Row::bin_size())..(index * Row::bin_size()),
            data.as_bin(&self.path)?,
        );

        let mut file = File::create(self.path.full().as_str())?;
        file.write_all(&bin)?;
        file.sync_all()
    }

    pub fn inserts(
        &mut self,
        index: usize,
        datas: impl Iterator<Item = Row>,
    ) -> std::io::Result<()> {
        let mut bin = self.read(0, None)?;

        bin.splice(
            (index * Row::bin_size())..(index * Row::bin_size()),
            datas
                .into_iter()
                .flat_map(|mut data| data.as_bin(&self.path))
                .flatten()
                .collect::<Vec<u8>>(),
        );

        let mut file = File::create(self.path.full().as_str())?;
        file.write_all(&bin)?;
        file.sync_all()
    }

    pub fn remove(&mut self, index: usize, len: Option<usize>) -> std::io::Result<()> {
        let data = self.read(0, None)?;

        let end = if let Some(len) = len {
            (index + len) * Row::bin_size()
        } else {
            data.len()
        };
        for to_delete in data[(index * Row::bin_size())..end]
            .chunks(Row::bin_size())
            .map(|row| Row::from_bin(row, &self.path))
        {
            to_delete?.delete(&self.path)?;
        }

        let mut file = File::create(self.path.full().as_str())?;
        if index > 0 {
            file.write_all(&data[..(index * Row::bin_size())])?;
        }
        if let Some(len) = len {
            file.write_all(&data[((index + len) * Row::bin_size())..])?;
        }
        file.sync_all()
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        remove_file(self.path.full())?;
        File::create(self.path.full())?;
        if self.path.folder().eq(&self.path.dir_path) && self.path.rel_file_path.eq("main.bin") {
            remove_dir_all(self.path.dyn_path())?;
            create_dir_all(self.path.dyn_path())?;
        }
        Ok(())
    }
}
