use std::fs::remove_dir_all;
use std::io;
use std::path::Path;

use crate::binary::Binary;
use crate::dyn_binary::DynanicBinary;
use crate::foreign::Foreign;
use crate::table::{IsRow, Table};

#[derive(Debug, Clone, PartialEq, Binary, IsRow)]
struct Client {
    #[PrimaryKey]
    id: usize,
    nom: DynanicBinary<usize, String>,
    entreprise: Foreign<usize, Entreprise>,
}

#[derive(Debug, Clone, PartialEq, Binary, IsRow)]
struct Entreprise {
    #[PrimaryKey]
    id: usize,
    nom: DynanicBinary<usize, String>,
}

pub fn test_table_get() {
    const CLIENTS_PATH: &str = "test/test_tableGetClients";
    const ENTREPRISES_PATH: &str = "test/test_tableGetEntreprises";

    let entreprises = [
        Entreprise {
            id: 1,
            nom: DynanicBinary::new(1, String::from("BigTech")),
        },
        Entreprise {
            id: 2,
            nom: DynanicBinary::new(2, String::from("Mine")),
        },
    ];

    let clients = [
        Client {
            id: 1,
            nom: DynanicBinary::new(1, String::from("Bob")),
            entreprise: Foreign::new(1),
        },
        Client {
            id: 2,
            nom: DynanicBinary::new(2, String::from("Fred")),
            entreprise: Foreign::new(1),
        },
        Client {
            id: 3,
            nom: DynanicBinary::new(3, String::from("Will")),
            entreprise: Foreign::new(2),
        },
    ];

    if Path::new(CLIENTS_PATH).exists() {
        remove_dir_all(CLIENTS_PATH).unwrap();
    }
    let table_clients =
        Table::<Client, usize>::new_default(CLIENTS_PATH, clients.clone().into_iter()).unwrap();

    if Path::new(ENTREPRISES_PATH).exists() {
        remove_dir_all(ENTREPRISES_PATH).unwrap();
    }
    let mut table_entreprises =
        Table::<Entreprise, usize>::new_default(ENTREPRISES_PATH, entreprises.clone().into_iter())
            .unwrap();

    for client in &clients {
        assert_eq!(
            table_clients
                .get(&client.id)
                .unwrap()
                .entreprise
                .data(&table_entreprises)
                .unwrap()
                .nom
                .data(),
            entreprises[*client.entreprise.id() - 1].nom.data()
        );
    }
    table_entreprises.clear().unwrap();
    for client in &clients {
        assert_eq!(
            table_clients
                .get(&client.id)
                .unwrap()
                .entreprise
                .data(&table_entreprises)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Other
        );
    }
}
#[test]
fn run_test_table_get() {
    test_table_get();
}
