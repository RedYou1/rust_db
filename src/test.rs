use std::{
    collections::HashMap,
    fs::{read_dir, remove_dir_all},
    path::Path,
};

use crate::{
    bin_file::{BinFile, Binary},
    dyn_binary::DynanicBinary,
};

#[derive(Debug, Clone, PartialEq, Binary)]
struct Test {
    a: [u32; 3],
    b: i128,
    c: DynanicBinary<u8, String>,
    d: f64,
    e: DynanicBinary<u8, HashMap<u32, u32>>,
}

fn nb_dyns(path: &str) -> usize {
    read_dir(format!("{path}/dyn"))
        .expect("dyn path doesn't exists")
        .count()
}

pub fn test_path() {
    const TABLE_PATH: &str = "test/testPath";
    let table = BinFile::<Test>::new(TABLE_PATH).expect("TABLE_PATH failed");
    assert_eq!(TABLE_PATH, table.path());
}
#[test]
fn run_test_path() {
    test_path();
}

pub fn test_default() {
    const TABLE_PATH: &str = "test/testDefault";
    let datas = vec![1, 6, 7];

    if Path::new(TABLE_PATH).exists() {
        remove_dir_all(TABLE_PATH).expect("TABLE_PATH doent exists");
    }
    let table = BinFile::<u8>::new_default(TABLE_PATH, datas.clone().into_iter())
        .expect("failed to new_default");
    assert_eq!(datas, table.gets(0, None).expect("failed to gets"));
}
#[test]
fn run_test_default() {
    test_default();
}

pub fn test1() {
    const TABLE_PATH: &str = "test/test1";
    let mut test1 = Test {
        a: [5, 255, 1_000_000],
        b: 1_000_000,
        c: DynanicBinary::new(0, String::from("Salut")),
        d: -1.1,
        e: DynanicBinary::new(1, [(1, 2), (2, 3)].into_iter().collect()),
    };
    let test2 = Test {
        a: [10000, 0, 5],
        b: -1,
        c: DynanicBinary::new(2, String::from("Wow")),
        d: 2.01,
        e: DynanicBinary::new(3, [(5, 4), (3, 2)].into_iter().collect()),
    };

    let mut table = BinFile::<Test>::new(TABLE_PATH).expect("failed to new");
    table.clear().expect("failed to clear");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(TABLE_PATH));

    assert_eq!("Salut", test1.c.data());
    *test1.c.mut_data() = String::from("Salut2");
    assert_eq!("Salut2", test1.c.data());

    table
        .inserts(0, [test2.clone(), test1.clone()].into_iter())
        .expect("failed inserts");
    assert_eq!(2, table.len().expect("failed len"));
    assert_eq!(4, nb_dyns(TABLE_PATH));
    assert_eq!(test2, table.get(0).expect("failed to get"));
    assert_eq!(test1, table.get(1).expect("failed to get"));

    let mut table = BinFile::<Test>::strict_new(TABLE_PATH);
    assert_eq!(2, table.len().expect("failed len"));
    assert_eq!(4, nb_dyns(TABLE_PATH));
    assert_eq!(test2, table.get(0).expect("failed to get"));
    assert_eq!(test1, table.get(1).expect("failed to get"));
    table.remove(0, Some(1)).expect("failed remove");
    assert_eq!(1, table.len().expect("failed len"));
    assert_eq!(2, nb_dyns(TABLE_PATH));
    assert_eq!(test1, table.get(0).expect("failed to get"));

    let mut table = BinFile::<Test>::strict_new(TABLE_PATH);
    assert_eq!(1, table.len().expect("failed len"));
    assert_eq!(2, nb_dyns(TABLE_PATH));
    assert_eq!(test1, table.get(0).expect("failed to get"));
    table.remove(0, None).expect("failed remove");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(TABLE_PATH));

    let table = BinFile::<Test>::strict_new(TABLE_PATH);
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(TABLE_PATH));
}
#[test]
fn run_test1() {
    test1();
}

pub fn test2() {
    const TABLE_PATH: &str = "test/test2";

    let mut a = DynanicBinary::new(b'a', b'b');
    *a.mut_data() = b'a';
    let b = DynanicBinary::new(b'b', b'b');
    let c = DynanicBinary::new(b'c', b'c');

    let mut table = BinFile::<DynanicBinary<u8, u8>>::new(TABLE_PATH).expect("failed new");
    table.clear().expect("failed clear");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(TABLE_PATH));

    table.insert(0, a.clone()).expect("failed insert");
    assert_eq!(1, table.len().expect("failed len"));
    assert_eq!(1, nb_dyns(TABLE_PATH));
    assert_eq!(a.clone(), table.get(0).expect("failed to get"));

    table
        .inserts(0, [b.clone(), c.clone()].into_iter())
        .expect("failed inserts");
    assert_eq!(3, nb_dyns(TABLE_PATH));
    assert_eq!(
        vec![b.clone(), c.clone(), a.clone()],
        table.gets(0, None).expect("failed to gets")
    );
    assert_eq!(
        vec![c.clone()],
        table.gets(1, Some(1)).expect("failed to gets")
    );
    assert_eq!(
        vec![b.clone()],
        table.gets(0, Some(1)).expect("failed to gets")
    );

    table.remove(1, Some(1)).expect("failed to remove");
    assert_eq!(2, nb_dyns(TABLE_PATH));
    assert_eq!(
        vec![b.clone(), a.clone()],
        table.gets(0, None).expect("failed to gets")
    );

    table.remove(0, None).expect("failed to remove");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(TABLE_PATH));
}
#[test]
fn run_test2() {
    test2();
}
