use std::{collections::HashMap, fmt::Debug, fs::read_dir};

use crate::{
    bd_path::BDPath,
    bin_file::{BaseBinFile, BinFile},
    binary::Binary,
    dyn_binary::DynanicBinary,
};

#[derive(Debug, Clone, PartialEq, Binary)]
pub struct Test {
    a: [u32; 3],
    b: i128,
    c: DynanicBinary<String>,
    d: f64,
    e: DynanicBinary<HashMap<u32, u32>>,
}

fn nb_dyns(path: &BDPath) -> std::io::Result<usize> {
    read_dir(path.dyn_path()).map(|f| f.count())
}

#[test]
pub fn test_path() {
    let path: BDPath = BDPath::new_main_str("test/testPath");
    let table = BinFile::<Test>::new(path.clone()).expect("TABLE_PATH failed");
    assert_eq!(path, *table.path());
}

#[test]
pub fn test1() {
    base_test1(BDPath::new_main_str("test/test1"), |path| {
        BinFile::new(path).expect("failed to new")
    });
}

pub fn base_test1<BinFile: BaseBinFile<Test>>(path: BDPath, new: impl Fn(BDPath) -> BinFile) {
    let mut test1 = Test {
        a: [5, 255, 1_000_000],
        b: 1_000_000,
        c: DynanicBinary::new(String::from("Salut")),
        d: -1.1,
        e: DynanicBinary::new([(1, 2), (2, 3)].into_iter().collect()),
    };
    let test2 = Test {
        a: [10000, 0, 5],
        b: -1,
        c: DynanicBinary::new(String::from("Wow")),
        d: 2.01,
        e: DynanicBinary::new([(5, 4), (3, 2)].into_iter().collect()),
    };

    let mut table = new(path.clone());
    table.clear().expect("failed to clear");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(&path).expect("nb_dyns"));

    assert_eq!("Salut", test1.c.data());
    *test1.c.mut_data() = String::from("Salut2");
    assert_eq!("Salut2", test1.c.data());

    table
        .inserts(0, &mut [test2.clone(), test1.clone()])
        .expect("failed inserts");
    assert_eq!(2, table.len().expect("failed len"));
    assert_eq!(4, nb_dyns(&path).expect("nb_dyns"));
    assert_eq!(test2, table.get(0).expect("failed to get"));
    assert_eq!(test1, table.get(1).expect("failed to get"));

    let mut table = new(path.clone());
    assert_eq!(2, table.len().expect("failed len"));
    assert_eq!(4, nb_dyns(&path).expect("nb_dyns"));
    assert_eq!(test2, table.get(0).expect("failed to get"));
    assert_eq!(test1, table.get(1).expect("failed to get"));
    table.remove(0, Some(1)).expect("failed remove");
    assert_eq!(1, table.len().expect("failed len"));
    assert_eq!(2, nb_dyns(&path).expect("nb_dyns"));
    assert_eq!(test1, table.get(0).expect("failed to get"));

    let mut table = new(path.clone());
    assert_eq!(1, table.len().expect("failed len"));
    assert_eq!(2, nb_dyns(&path).expect("nb_dyns"));
    assert_eq!(test1, table.get(0).expect("failed to get"));
    table.remove(0, None).expect("failed remove");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(&path).expect("nb_dyns"));

    let table = new(path.clone());
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(&path).expect("nb_dyns"));
}

#[test]
pub fn test2() {
    base_test2(BDPath::new_main_str("test/test2"), |path| {
        BinFile::new(path).expect("failed to new")
    });
}

pub fn base_test2<BinFile: BaseBinFile<DynanicBinary<u8>>>(
    path: BDPath,
    new: impl Fn(BDPath) -> BinFile,
) {
    let mut a = DynanicBinary::new(b'b');
    *a.mut_data() = b'a';
    let b = DynanicBinary::new(b'b');
    let c = DynanicBinary::new(b'c');

    let mut table = new(path.clone());
    table.clear().expect("failed clear");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(&path).expect("nb_dyns"));

    table.insert(0, &mut a).expect("failed insert");
    assert_eq!(1, table.len().expect("failed len"));
    assert_eq!(1, nb_dyns(&path).expect("nb_dyns"));
    assert_eq!(a.clone(), table.get(0).expect("failed to get"));

    table
        .inserts(0, &mut [b.clone(), c.clone()])
        .expect("failed inserts");
    assert_eq!(3, nb_dyns(&path).expect("nb_dyns"));
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
    assert_eq!(2, nb_dyns(&path).expect("nb_dyns"));
    assert_eq!(
        vec![b.clone(), a.clone()],
        table.gets(0, None).expect("failed to gets")
    );

    table.remove(0, None).expect("failed to remove");
    assert_eq!(0, table.len().expect("failed len"));
    assert_eq!(0, nb_dyns(&path).expect("nb_dyns"));
}
