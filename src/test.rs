use std::{
    fs::{read_dir, remove_dir_all},
    path::Path,
};

use crate::{
    dyn_binary::DynanicBinary,
    table::{Binary, Table},
};

#[derive(Debug, Clone, PartialEq, Binary)]
struct Test {
    a: [u32; 3],
    b: i128,
    c: DynanicBinary<u8, String>,
    d: f64,
}

fn nb_dyns(path: &str) -> usize {
    read_dir(format!("{path}/dyn")).unwrap().count()
}

#[test]
fn test_path() {
    const TABLE_PATH: &str = "test/testPath";
    let table = Table::<Test>::new(TABLE_PATH).unwrap();
    assert_eq!(TABLE_PATH, table.path());
}

#[test]
fn test_default() {
    const TABLE_PATH: &str = "test/testDefault";
    let datas = vec![1, 6, 7];

    if Path::new(TABLE_PATH).exists() {
        remove_dir_all(TABLE_PATH).unwrap();
    }
    let table = Table::<u8>::new_default(TABLE_PATH, datas.clone().into_iter()).unwrap();
    assert_eq!(datas, table.gets(0, None).unwrap());
}

#[test]
fn test1() {
    const TABLE_PATH: &str = "test/test1";
    let mut test1 = Test {
        a: [5, 255, 1000000],
        b: 1000000,
        c: DynanicBinary::new(0, String::from("Salut")),
        d: -1.1,
    };
    let test2 = Test {
        a: [10000, 0, 5],
        b: -1,
        c: DynanicBinary::new(1, String::from("Wow")),
        d: 2.01,
    };

    let mut table = Table::<Test>::new(TABLE_PATH).unwrap();
    table.clear().unwrap();
    assert_eq!(0, table.len().unwrap());
    assert_eq!(0, nb_dyns(TABLE_PATH));

    assert_eq!("Salut", test1.c.data());
    *test1.c.mut_data() = String::from("Salut2");
    assert_eq!("Salut2", test1.c.data());

    table
        .inserts(0, [test2.clone(), test1.clone()].into_iter())
        .unwrap();
    assert_eq!(2, table.len().unwrap());
    assert_eq!(2, nb_dyns(TABLE_PATH));
    assert_eq!(test2, table.get(0).unwrap());
    assert_eq!(test1, table.get(1).unwrap());

    let mut table = Table::<Test>::strict_new(TABLE_PATH);
    assert_eq!(2, table.len().unwrap());
    assert_eq!(2, nb_dyns(TABLE_PATH));
    assert_eq!(test2, table.get(0).unwrap());
    assert_eq!(test1, table.get(1).unwrap());
    table.remove(0, Some(1)).unwrap();
    assert_eq!(1, table.len().unwrap());
    assert_eq!(1, nb_dyns(TABLE_PATH));
    assert_eq!(test1, table.get(0).unwrap());

    let mut table = Table::<Test>::strict_new(TABLE_PATH);
    assert_eq!(1, table.len().unwrap());
    assert_eq!(1, nb_dyns(TABLE_PATH));
    assert_eq!(test1, table.get(0).unwrap());
    table.remove(0, None).unwrap();
    assert_eq!(0, table.len().unwrap());
    assert_eq!(0, nb_dyns(TABLE_PATH));

    let table = Table::<Test>::strict_new(TABLE_PATH);
    assert_eq!(0, table.len().unwrap());
    assert_eq!(0, nb_dyns(TABLE_PATH));
}

#[test]
fn test2() {
    const TABLE_PATH: &str = "test/test2";

    let mut a = DynanicBinary::new(b'a', b'b');
    *a.mut_data() = b'a';
    let b = DynanicBinary::new(b'b', b'b');
    let c = DynanicBinary::new(b'c', b'c');

    let mut table = Table::<DynanicBinary<u8, u8>>::new(TABLE_PATH).unwrap();
    table.clear().unwrap();
    assert_eq!(0, table.len().unwrap());
    assert_eq!(0, nb_dyns(TABLE_PATH));

    table.insert(0, a.clone()).unwrap();
    assert_eq!(1, table.len().unwrap());
    assert_eq!(1, nb_dyns(TABLE_PATH));
    assert_eq!(a.clone(), table.get(0).unwrap());

    table
        .inserts(0, [b.clone(), c.clone()].into_iter())
        .unwrap();
    assert_eq!(3, nb_dyns(TABLE_PATH));
    assert_eq!(
        vec![b.clone(), c.clone(), a.clone()],
        table.gets(0, None).unwrap()
    );
    assert_eq!(vec![c.clone()], table.gets(1, Some(1)).unwrap());
    assert_eq!(vec![b.clone()], table.gets(0, Some(1)).unwrap());

    table.remove(1, Some(1)).unwrap();
    assert_eq!(2, nb_dyns(TABLE_PATH));
    assert_eq!(vec![b.clone(), a.clone()], table.gets(0, None).unwrap());

    table.remove(0, None).unwrap();
    assert_eq!(0, table.len().unwrap());
    assert_eq!(0, nb_dyns(TABLE_PATH));
}
