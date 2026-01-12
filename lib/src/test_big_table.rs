use std::fs::remove_dir_all;

use crate::prelude::*;

#[derive(Clone, PartialEq, Table)]
pub struct A {
    #[PrimaryKey]
    pub id: usize,
}

#[test]
pub fn test_index() {
    const PATH: &str = "test/big";
    if Path::new(PATH).exists() {
        remove_dir_all(PATH).expect("PATH already exists");
    }
    let mut table = CachedTableFile::new(PATH.to_owned()).expect("failed to create table_clients");

    for i in 0..300 {
        while !table
            .insert(&mut A {
                id: rand::random::<u64>() as usize,
            })
            .expect("filled")
        {}
        println!("inserted {i}");
    }
    assert_eq!(300, table.len().expect("len"));

    // let mut i: usize = 0;
    // loop {
    //     if table
    //         .insert(&mut A {
    //             id: rand::random::<u64>() as usize,
    //         })
    //         .expect("filled")
    //     {
    //         i += 1;
    //     }
    //     if i == 1_000 {
    //         break;
    //     }
    // }
    // assert_eq!(1_000, table.len().expect("OK"));
}
