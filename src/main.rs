mod binary;

use binary::Binary;
use rust_db::Binary;
use std::{
    fs::File,
    io::{Read, Write},
};

#[derive(Debug, Binary)]
struct Row {
    a1: u8,
    b1: u16,
    c1: u32,
    d1: u64,
    e1: u128,
    a2: i8,
    b2: i16,
    c2: i32,
    d2: i64,
    e2: i128,
    q1: bool,
    q2: char,
}

fn read_one<T>(filename: &str) -> std::io::Result<T>
where
    T: Binary,
{
    let mut file = File::open(filename)?;
    let metadata = file.metadata()?;
    assert!(T::bin_size() <= metadata.len() as usize);
    let mut result = vec![0 as u8; T::bin_size()];
    file.read(&mut result)?;
    Ok(result
        .chunks(T::bin_size())
        .map(T::from_bin)
        .next()
        .unwrap())
}

fn read_all<T>(filename: &str) -> std::io::Result<Vec<T>>
where
    T: Binary,
{
    let mut file = File::open(filename)?;
    let metadata = file.metadata()?;
    let mut result = vec![0 as u8; metadata.len() as usize];
    file.read(&mut result)?;
    Ok(result
        .chunks(T::bin_size())
        .map(T::from_bin)
        .collect::<Vec<T>>())
}

fn write_all<T>(filename: &str, data: &[T]) -> std::io::Result<()>
where
    T: Binary,
{
    let mut file = File::create(filename)?;
    let bin = data
        .into_iter()
        .flat_map(|data| data.into_bin())
        .collect::<Vec<u8>>();
    file.write(&bin)?;
    Ok(())
}

fn write_one<T>(filename: &str, data: &T) -> std::io::Result<()>
where
    T: Binary,
{
    let mut file = File::create(filename)?;
    let bin = data.into_bin();
    file.write(&bin)?;
    Ok(())
}

fn gen(a: u8) -> Row {
    Row {
        a1: u8::from_bin(&[a]),
        b1: u16::from_bin(&[a, a]),
        c1: u32::from_bin(&[a, a, a, a]),
        d1: u64::from_bin(&[a, a, a, a, a, a, a, a]),
        e1: u128::from_bin(&[a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a]),
        a2: i8::from_bin(&[a]),
        b2: i16::from_bin(&[a, a]),
        c2: i32::from_bin(&[a, a, a, a]),
        d2: i64::from_bin(&[a, a, a, a, a, a, a, a]),
        e2: i128::from_bin(&[a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a]),
        q1: bool::from_bin(&[a]),
        q2: char::from_bin(&[a]),
    }
}

fn main() {
    const FILE_NAME: &str = "test/1.bin";
    let a = gen(0);
    println!("{a:?}");
    write_one(FILE_NAME, &a).unwrap();
    let b = read_one::<Row>(FILE_NAME).unwrap();
    println!("{b:?}");

    const FILE_NAME2: &str = "test/2.bin";
    let a = [gen(255), gen(1), gen(2), gen(3)];
    println!("{a:?}");
    write_all(FILE_NAME2, &a).unwrap();
    let b = read_all::<Row>(FILE_NAME2).unwrap();
    println!("{b:?}");
}
