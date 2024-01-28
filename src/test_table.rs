use std::collections::HashMap;
use std::fs::remove_dir_all;
use std::io;
use std::path::Path;

use crate::binary::Binary;
use crate::foreign::Foreign;
use crate::row_binary::RowBinary;
use crate::row_dyn_binary::RowDynanicBinary;
use crate::table::{Table, TableRow};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, TableRow)]
struct Client {
    #[PrimaryKey]
    id: usize,
    nom: RowDynanicBinary<usize, String>,
    entreprise: Foreign<usize, Entreprise>,
}

#[derive(Debug, Clone, PartialEq, TableRow)]
struct Entreprise {
    #[PrimaryKey]
    id: usize,
    nom: RowDynanicBinary<usize, String>,
    employe: RowDynanicBinary<usize, HashMap<usize, [char; 4]>>,
}

pub fn test_table_get() {
    const CLIENTS_PATH: &str = "test/test_tableGetClients";
    const ENTREPRISES_PATH: &str = "test/test_tableGetEntreprises";

    let entreprises = [
        Entreprise {
            id: 1,
            nom: RowDynanicBinary::new(String::from("BigTech")),
            employe: RowDynanicBinary::new(
                [
                    (0, ['M', 'L', 'P', 'X']),
                    (1, ['K', 'H', 'E', 'A']),
                    (2, ['J', 'Q', 'V', 'Z']),
                ]
                .into_iter()
                .collect(),
            ),
        },
        Entreprise {
            id: 2,
            nom: RowDynanicBinary::new(String::from("Mine")),
            employe: RowDynanicBinary::new([(0, ['U', 'R', 'W', 'S'])].into_iter().collect()),
        },
    ];

    let clients = [
        Client {
            id: 1,
            nom: RowDynanicBinary::new(String::from("Bob")),
            entreprise: Foreign::new(1),
        },
        Client {
            id: 2,
            nom: RowDynanicBinary::new(String::from("Fred")),
            entreprise: Foreign::new(1),
        },
        Client {
            id: 3,
            nom: RowDynanicBinary::new(String::from("Will")),
            entreprise: Foreign::new(2),
        },
    ];

    if Path::new(CLIENTS_PATH).exists() {
        remove_dir_all(CLIENTS_PATH).expect("CLIENTS_PATH already exists");
    }
    let table_clients = Table::new_default(CLIENTS_PATH, clients.clone().into_iter())
        .expect("failed to create table_clients");

    if Path::new(ENTREPRISES_PATH).exists() {
        remove_dir_all(ENTREPRISES_PATH).expect("ENTREPRISES_PATH already exists");
    }
    let mut table_entreprises =
        Table::new(ENTREPRISES_PATH).expect("failed to create table_entreprises");
    table_entreprises
        .inserts(entreprises.clone().into_iter())
        .expect("failed to insert entreprises");

    for client in &clients {
        assert_eq!(
            table_clients
                .get(&client.id)
                .expect("client doesnt exists")
                .entreprise
                .data(&table_entreprises)
                .expect("entreprise doesnt exists")
                .nom
                .data(),
            table_entreprises
                .get(client.entreprise.id())
                .expect("entreprise doesnt exists")
                .nom
                .data()
        );
    }
    table_entreprises
        .clear()
        .expect("failed to clear entreprise");
    for client in &clients {
        assert_eq!(
            table_clients
                .get(&client.id)
                .expect("client doesnt exists")
                .entreprise
                .data(&table_entreprises)
                .expect_err("entreprise does exists")
                .kind(),
            io::ErrorKind::Other
        );
    }
}
#[test]
fn run_test_table_get() {
    test_table_get();
}
