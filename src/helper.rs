use crate::binary::Binary;
use std::{
    fs::File,
    io::{self, Read},
};

pub fn remove_errors<T>(items: impl Iterator<Item = io::Result<T>>) -> io::Result<Vec<T>> {
    let mut result = vec![];
    for (i, item) in items.enumerate() {
        match item {
            Ok(item) => result.insert(i, item),
            Err(err) => return Err(err),
        }
    }
    Ok(result)
}

pub fn flat_remove_errors<T>(
    items: impl Iterator<Item = io::Result<Vec<T>>>,
) -> io::Result<Vec<T>> {
    let mut result = vec![];
    for (i, item) in items.enumerate() {
        match item {
            Ok(item) => result.insert(i, item),
            Err(err) => return Err(err),
        }
    }
    Ok(result.into_iter().flatten().collect())
}

pub fn read_all<Row>(path: &str) -> std::io::Result<Vec<Row>>
where
    Row: Binary,
{
    let mut file = File::open(format!("{path}/main.bin"))?;
    let mut result = vec![0 as u8; file.metadata()?.len() as usize];
    file.read(&mut result)?;

    remove_errors(
        result
            .chunks(Row::bin_size())
            .map(|row| Row::from_bin(row, path)),
    )
}
