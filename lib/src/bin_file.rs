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

pub trait BaseBinFile<Row>: Sized {
    fn new(path: BDPath) -> io::Result<Self>;
    fn path(&self) -> &BDPath;
    fn get(&self, index: usize) -> io::Result<Row>;
    fn gets(&self, index: usize, len: Option<usize>) -> io::Result<Vec<Row>>;
    fn is_empty(&self) -> io::Result<bool>;
    fn len(&self) -> io::Result<usize>;
    fn insert(&mut self, index: usize, data: &mut Row) -> std::io::Result<()>;
    fn inserts(&mut self, index: usize, datas: &mut [Row]) -> std::io::Result<()>;
    fn remove(&mut self, index: usize, len: Option<usize>) -> std::io::Result<()>;
    fn clear(&mut self) -> std::io::Result<()>;
}

impl<Row: Binary> BinFile<Row> {
    pub const fn path(&self) -> &BDPath {
        &self.path
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
            if len == 0 {
                return Err(Error::other("must read at least 1 element"));
            }
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

    fn file_len(&self) -> io::Result<usize> {
        Ok(fs::metadata(self.path.full())?.len() as usize)
    }

    fn base_insert(&mut self, index: usize, data: &[u8]) -> std::io::Result<()> {
        if index == self.len()? {
            let mut file = File::options()
                .append(true)
                .create(true)
                .open(self.path.full().as_str())?;
            file.write_all(data)?;
            return file.sync_all();
        }
        let temp = BDPath {
            dir_path: self.path.dir_path.clone(),
            rel_file_path: self.path.rel_file_path.replace(".", ".temp."),
        };

        let mut buf = [0_u8; 1028];

        {
            let mut new = File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(temp.full().as_str())?;
            let mut original = File::options().read(true).open(self.path.full().as_str())?;

            let (start, end) = {
                let mut i: usize = 0;
                let until = Row::bin_size() * index;
                loop {
                    let n = original.read(&mut buf)?;
                    if n == 0 {
                        break (0, 0);
                    }
                    let wanted = until - i;
                    if n >= wanted {
                        new.write_all(&buf[..wanted])?;
                        break (wanted, n);
                    }
                    new.write_all(&buf[..n])?;
                    i += n;
                }
            };
            new.write_all(data)?;
            if start != end {
                new.write_all(&buf[start..end])?;
            }
            loop {
                let n = original.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                new.write_all(&buf[..n])?;
            }
            new.sync_all()?;
        }
        fs::rename(temp.full().as_str(), self.path.full().as_str())
    }
}

impl<Row: Binary> BaseBinFile<Row> for BinFile<Row> {
    fn new(path: BDPath) -> std::io::Result<Self> {
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

    fn path(&self) -> &BDPath {
        &self.path
    }

    fn get(&self, index: usize) -> io::Result<Row> {
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

    fn gets(&self, index: usize, len: Option<usize>) -> io::Result<Vec<Row>> {
        self.read(index, len)?
            .chunks(Row::bin_size())
            .map(|row| Row::from_bin(row, &self.path))
            .collect()
    }

    fn is_empty(&self) -> io::Result<bool> {
        Ok(fs::metadata(self.path.full())?.len() == 0)
    }

    fn len(&self) -> io::Result<usize> {
        Ok(fs::metadata(self.path.full())?.len() as usize / Row::bin_size())
    }

    fn insert(&mut self, index: usize, data: &mut Row) -> std::io::Result<()> {
        self.base_insert(index, &data.as_bin(&self.path)?)
    }

    fn inserts(&mut self, index: usize, datas: &mut [Row]) -> std::io::Result<()> {
        self.base_insert(
            index,
            &datas
                .iter_mut()
                .flat_map(|data| data.as_bin(&self.path))
                .flatten()
                .collect::<Vec<u8>>(),
        )
    }

    fn remove(&mut self, index: usize, len: Option<usize>) -> std::io::Result<()> {
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

    fn clear(&mut self) -> std::io::Result<()> {
        remove_file(self.path.full())?;
        File::create(self.path.full())?;
        if self.path.folder().eq(&self.path.dir_path) && self.path.rel_file_path.eq("main.bin") {
            remove_dir_all(self.path.dyn_path())?;
            create_dir_all(self.path.dyn_path())?;
        }
        Ok(())
    }
}
