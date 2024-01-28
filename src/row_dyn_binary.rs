use std::{
    fmt::Display,
    fs::{remove_file, File},
    io::{self, Read, Write},
    marker::PhantomData,
};

use crate::{binary::Binary, dyn_binary::AsBinary, row_binary::RowBinary};

#[derive(Debug, Clone, PartialEq)]
pub struct RowDynanicBinary<ID, DATA>
where
    ID: Binary + Display,
    DATA: AsBinary,
{
    id: PhantomData<ID>,
    data: DATA,
}

impl<ID: Binary + Display, DATA: AsBinary> RowDynanicBinary<ID, DATA> {
    pub const fn new(data: DATA) -> Self {
        RowDynanicBinary {
            id: PhantomData,
            data,
        }
    }

    pub const fn data(&self) -> &DATA {
        &self.data
    }

    pub fn mut_data(&mut self) -> &mut DATA {
        &mut self.data
    }
}

impl<ID: Binary + Display, DATA: AsBinary> RowBinary<ID> for RowDynanicBinary<ID, DATA> {
    fn from_row_bin(_: &[u8], id: &ID, path: &str) -> io::Result<Self> {
        let mut file = File::open(format!("{path}/dyn/{id}.bin"))?;
        let mut result = vec![0; file.metadata()?.len() as usize];
        file.read_exact(&mut result)?;
        Ok(RowDynanicBinary {
            id: PhantomData,
            data: DATA::from_as_bin(result, path)?,
        })
    }

    fn as_row_bin(&self, id: &ID, path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::create(format!("{path}/dyn/{id}.bin"))?;
        file.write_all(&self.data.as_as_bin(path)?)?;
        file.sync_all()?;

        Ok(vec![])
    }

    fn row_bin_size(_: PhantomData<ID>) -> usize {
        0
    }

    fn row_delete(&self, id: &ID, path: &str) -> io::Result<()> {
        remove_file(format!("{path}/dyn/{id}.bin"))
    }
}
