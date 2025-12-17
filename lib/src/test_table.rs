use std::collections::HashMap;
use std::fs::remove_dir_all;
use std::io;
use std::path::Path;

use rust_db::binary::Binary;
use rust_db::dyn_binary::DynanicBinary;
use rust_db::foreign::Foreign;
use rust_db::table::{Table, TableRow};

#[derive(Debug, Clone, PartialEq, TableRow)]
struct Client {
    #[PrimaryKey]
    id: usize,
    nom: DynanicBinary<String>,
    entreprise: Foreign<usize, Entreprise>,
}

#[derive(Debug, Clone, PartialEq, TableRow)]
struct Entreprise {
    #[PrimaryKey]
    id: usize,
    nom: DynanicBinary<String>,
    employe: DynanicBinary<HashMap<u8, [char; 4]>>,
}

pub fn test_table_get() {
    const CLIENTS_PATH: &str = "test/test_tableGetClients";
    const ENTREPRISES_PATH: &str = "test/test_tableGetEntreprises";

    let entreprises = [
        Entreprise {
            id: 1,
            nom: DynanicBinary::new(String::from("BigTech")),
            employe: DynanicBinary::new(
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
            nom: DynanicBinary::new(String::from("Mine")),
            employe: DynanicBinary::new([(0, ['U', 'R', 'W', 'S'])].into_iter().collect()),
        },
    ];

    let clients = [
        Client {
            id: 1,
            nom: DynanicBinary::new(String::from("Bob")),
            entreprise: Foreign::new(1),
        },
        Client {
            id: 2,
            nom: DynanicBinary::new(String::from("Fred")),
            entreprise: Foreign::new(1),
        },
        Client {
            id: 3,
            nom: DynanicBinary::new(String::from("Will")),
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

    for (a, b) in entreprises
        .iter()
        .zip(table_entreprises.get_all().expect("gets entreprises"))
    {
        assert_eq!(a.id, b.id);
        assert_eq!(a.nom, b.nom);
        assert_eq!(a.employe.data().len(), b.employe.data().len());
        assert!(b.nom.id().is_some());
        assert!(b.employe.id().is_some());
        for a in a.employe.data() {
            let b = b.employe.data()[a.0];
            assert!(a.1[0].eq(&b[0]));
            assert!(a.1[1].eq(&b[1]));
            assert!(a.1[2].eq(&b[2]));
            assert!(a.1[3].eq(&b[3]));
        }
    }

    for client in table_clients.get_all().expect("client doesnt exists") {
        assert!(client.nom.id().is_some());
        assert_eq!(
            client
                .entreprise
                .data(&table_entreprises)
                .expect("entreprise doesnt exists")
                .nom,
            table_entreprises
                .get(client.entreprise.id())
                .expect("entreprise doesnt exists")
                .nom
        );
    }
    table_entreprises
        .clear()
        .expect("failed to clear entreprise");
    for client in table_clients.get_all().expect("client doesnt exists") {
        assert_eq!(
            client
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
