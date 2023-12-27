mod binary;
mod dyn_binary;
mod table;
#[cfg(test)]
mod test;

use dyn_binary::DynanicBinary;
use table::{Binary, Table};

#[derive(Debug, Clone, PartialEq, Binary)]
struct Row {
    name: DynanicBinary<u8, String>,
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

fn gen(a: u8, path: &str) -> Row {
    Row {
        name: DynanicBinary::new(a, a.to_string()),
        a1: u8::from_bin(&[a], path),
        b1: u16::from_bin(&[a, a], path),
        c1: u32::from_bin(&[a, a, a, a], path),
        d1: u64::from_bin(&[a, a, a, a, a, a, a, a], path),
        e1: u128::from_bin(&[a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a], path),
        a2: i8::from_bin(&[a], path),
        b2: i16::from_bin(&[a, a], path),
        c2: i32::from_bin(&[a, a, a, a], path),
        d2: i64::from_bin(&[a, a, a, a, a, a, a, a], path),
        e2: i128::from_bin(&[a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a], path),
        q1: [
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
            bool::from_bin(&[a], path),
        ],
        q2: char::from_bin(&[a], path),
    }
}

fn main() {
    let mut table = Table::<Row>::new("test/1").unwrap();
    assert_eq!("test/1", table.path());

    while table.len() > 0 {
        println!("1: Removing first row {}", table.len());
        table.remove(0).unwrap();
    }
    assert_eq!(1, table.nb_files());

    for i in 0..=5 {
        let mut row = gen(i * 51, table.path());
        assert_eq!(&(i * 51).to_string(), row.name.data());
        *row.name.mut_data() = format!("WOW{}", i * 51);
        assert_eq!(&format!("WOW{}", i * 51), row.name.data());
        table.insert(0, row.clone()).unwrap();
        assert_eq!(row, *table.get(0).unwrap());
    }

    println!("table1 size {}", table.len());

    let mut table2 = Table::<Row>::new("test/2").unwrap();

    while table2.len() > 0 {
        println!("2: Removing first row {}", table2.len());
        table2.remove(0).unwrap();
    }

    for (index, data) in table.datas().into_iter().enumerate() {
        table2.insert(index, data.clone()).unwrap();
    }

    println!("table2 size {}", table.len());

    assert_eq!(table.datas(), table.datas());

    println!("Success");
}
