use crate::{
    bd_path::BDPath,
    binary::Binary,
    prelude::BaseBinFile,
    table::{SpecificTableFile, Table, TableGet},
};

#[derive(Debug, Clone)]
pub struct Foreign<Row: Table> {
    id: Row::ID,
}

impl<Row: Table> PartialEq for Foreign<Row> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Row: Table> PartialOrd for Foreign<Row> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(other.id())
    }
}

impl<Row: Table> Binary for Foreign<Row> {
    fn from_bin(data: &[u8], path: &BDPath) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Foreign {
            id: Row::ID::from_bin(data, path)?,
        })
    }

    fn as_bin(&mut self, path: &BDPath) -> std::io::Result<Vec<u8>> {
        self.id.as_bin(path)
    }

    fn bin_size() -> usize {
        Row::ID::bin_size()
    }

    fn delete(&self, _: &BDPath) -> std::io::Result<()> {
        Ok(())
    }
}

impl<Row: Table> Foreign<Row> {
    pub const fn new(id: Row::ID) -> Self {
        Foreign { id }
    }

    pub const fn id(&self) -> &Row::ID {
        &self.id
    }

    pub fn data<BinFile: BaseBinFile<Row>>(
        &self,
        table: &SpecificTableFile<Row, BinFile>,
    ) -> TableGet<Row> {
        table.get_by_id(&self.id)
    }
}
