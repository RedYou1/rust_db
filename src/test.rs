use std::{fs::File, io::Write};

use crate::table::{Binary, Table};

#[derive(Debug, Clone, Binary)]
struct Test {
    a: [u32; 3],
    b: i128,
}

#[test]
fn test_path() {
    const TABLE_PATH: &str = "test/testPath.bin";
    let table = Table::<Test>::new(TABLE_PATH).unwrap();
    assert_eq!(TABLE_PATH, table.path());
}

#[test]
fn test1() {
    const TABLE_PATH: &str = "test/test1.bin";
    let test1 = Test {
        a: [5, 255, 1000000],
        b: 1000000,
    };
    let test2 = Test {
        a: [10000, 0, 5],
        b: -1,
    };
    File::create(TABLE_PATH).unwrap().write_all(&[]).unwrap();

    let mut table = Table::<Test>::new(TABLE_PATH).unwrap();
    assert_eq!(0, table.len());
    table.insert(0, test1.clone()).unwrap();
    table.insert(0, test2.clone()).unwrap();
    assert_eq!(2, table.len());
    assert_eq!(test2.into_bin(), table.get(0).unwrap().into_bin());
    assert_eq!(test1.into_bin(), table.get(1).unwrap().into_bin());

    let mut table = Table::<Test>::new(TABLE_PATH).unwrap();
    assert_eq!(2, table.len());
    assert_eq!(test2.into_bin(), table.get(0).unwrap().into_bin());
    assert_eq!(test1.into_bin(), table.get(1).unwrap().into_bin());
    table.remove(0).unwrap();
    assert_eq!(1, table.len());
    assert_eq!(test1.into_bin(), table.get(0).unwrap().into_bin());

    let mut table = Table::<Test>::new(TABLE_PATH).unwrap();
    assert_eq!(1, table.len());
    assert_eq!(test1.into_bin(), table.get(0).unwrap().into_bin());
    table.remove(0).unwrap();
    assert_eq!(0, table.len());

    let table = Table::<Test>::new(TABLE_PATH).unwrap();
    assert_eq!(0, table.len());
}
