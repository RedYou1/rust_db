mod binary;
mod table;
#[cfg(test)]
mod test;

use binary::Binary;
use rust_db::Binary;

use crate::table::Table;

#[derive(Debug, Clone, Binary)]
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
    q1: [bool; 8],
    q2: char,
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
        q1: [
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
            bool::from_bin(&[a]),
        ],
        q2: char::from_bin(&[a]),
    }
}

fn main() {
    let mut table = Table::<Row>::new("test/1.bin").unwrap();
    assert_eq!("test/1.bin", table.path());

    while table.len() > 0 {
        println!("1: Removing first row {}", table.len());
        table.remove(0).unwrap();
    }

    for i in 0..=5 {
        let row = gen(i * 51);
        table.insert(0, row.clone()).unwrap();
        assert_eq!(row.into_bin(), table.get(0).unwrap().into_bin());
    }

    println!("table1 size {}", table.len());

    let mut table2 = Table::<Row>::new("test/2.bin").unwrap();

    while table2.len() > 0 {
        println!("2: Removing first row {}", table2.len());
        table2.remove(0).unwrap();
    }

    for (index, data) in table.datas().into_iter().enumerate() {
        table2.insert(index, data.clone()).unwrap();
    }

    println!("table2 size {}", table.len());

    assert_eq!(
        table
            .datas()
            .into_iter()
            .flat_map(|row| row.into_bin())
            .collect::<Vec<u8>>(),
        table
            .datas()
            .into_iter()
            .flat_map(|row| row.into_bin())
            .collect::<Vec<u8>>()
    );

    println!("Success");
}
