use std::fs::remove_dir_all;

use crate::prelude::*;

#[derive(Clone, PartialEq, Table)]
pub struct A {
    #[PrimaryKey]
    pub id: usize,
    #[Unique]
    pub rel: Foreign<B>,
}

#[derive(Clone, PartialEq, Table)]
pub struct B {
    #[PrimaryKey]
    pub id: usize,
    #[Index]
    pub rel: Foreign<A>,
}

#[test]
pub fn test_index() {
    const PATH_A: &str = "test/index_A";
    const PATH_B: &str = "test/index_B";
    if Path::new(PATH_A).exists() {
        remove_dir_all(PATH_A).expect("PATH_A already exists");
    }
    let mut table_a =
        CachedTableFile::new(PATH_A.to_owned()).expect("failed to create table_clients");

    if Path::new(PATH_B).exists() {
        remove_dir_all(PATH_B).expect("PATH_B already exists");
    }
    let mut table_b =
        CachedTableFile::new(PATH_B.to_owned()).expect("failed to create table_clients");

    assert!(
        table_a
            .insert(&mut A {
                id: 1,
                rel: Foreign::new(1),
            })
            .expect("OK")
    );

    assert!(
        !table_a
            .insert(&mut A {
                id: 2,
                rel: Foreign::new(1),
            })
            .expect("OK")
    );

    assert!(
        table_b
            .insert(&mut B {
                id: 1,
                rel: Foreign::new(1),
            })
            .expect("OK")
    );

    assert!(
        table_b
            .insert(&mut B {
                id: 2,
                rel: Foreign::new(1),
            })
            .expect("allowed duplicate foreign")
    );

    assert_eq!(1, table_a.len().expect("OK"));
    assert_eq!(2, table_b.len().expect("OK"));

    let TableGet::Found(b2) = table_b.get_by_id(&2) else {
        panic!("OK")
    };
    assert_eq!(2, b2.id);
    let TableGet::Found(a1) = b2.rel.data(&table_a) else {
        panic!("OK")
    };
    assert_eq!(1, a1.id);
    let TableGet::Found(b1) = a1.rel.data(&table_b) else {
        panic!("OK")
    };
    assert_eq!(1, b1.id);
    let TableGet::Found(a1) = b1.rel.data(&table_a) else {
        panic!("OK")
    };
    assert_eq!(1, a1.id);

    let TableGet::Found(Some(a1)) = table_a.get_by_rel(&Foreign::new(1)) else {
        panic!("OK")
    };
    assert_eq!(1, a1.id);
    let TableGet::Found(bs) = table_b.get_by_rel(&Foreign::new(1)) else {
        panic!("OK")
    };
    assert_eq!(2, bs.len());
}
