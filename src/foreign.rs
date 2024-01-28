use std::{io, marker::PhantomData};

use crate::{
    binary::Binary,
    table::{Table, TableRow},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Foreign<ID, Row>
where
    ID: Binary + PartialEq,
    Row: TableRow<ID>,
{
    id: ID,
    phantom_data: PhantomData<Row>,
}

impl<ID, Row> Binary for Foreign<ID, Row>
where
    ID: Binary + PartialEq,
    Row: TableRow<ID>,
{
    fn from_bin(data: &[u8], path: &str) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Foreign {
            id: ID::from_bin(data, path)?,
            phantom_data: PhantomData,
        })
    }

    fn as_bin(&self, path: &str) -> std::io::Result<Vec<u8>> {
        self.id.as_bin(path)
    }

    fn bin_size() -> usize {
        ID::bin_size()
    }

    fn delete(&self, _: &str) -> std::io::Result<()> {
        Ok(())
    }
}

impl<ID, Row> Foreign<ID, Row>
where
    ID: Binary + PartialEq,
    Row: TableRow<ID>,
{
    pub const fn new(id: ID) -> Self {
        Foreign {
            id,
            phantom_data: PhantomData,
        }
    }

    pub const fn id(&self) -> &ID {
        &self.id
    }

    pub fn data(&self, table: &Table<Row, ID>) -> io::Result<Row> {
        table.get(&self.id)
    }
}
