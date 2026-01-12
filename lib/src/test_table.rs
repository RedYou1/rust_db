use std::collections::HashMap;
use std::fs::remove_dir_all;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Table)]
struct Client {
    #[PrimaryKey]
    id: usize,
    #[Index]
    nom: DynanicBinary<String>,
    entreprise: Foreign<Entreprise>,
}

#[derive(Debug, Clone, PartialEq, Table)]
struct Entreprise {
    #[PrimaryKey]
    id: usize,
    nom: DynanicBinary<String>,
    employe: DynanicBinary<HashMap<u8, [char; 4]>>,
}

#[expect(clippy::too_many_lines)]
#[test]
pub fn test_table_get() {
    const CLIENTS_PATH: &str = "test/test_tableGetClients";
    const ENTREPRISES_PATH: &str = "test/test_tableGetEntreprises";

    let mut entreprises = [
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

    let mut clients = [
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
    let mut table_clients =
        CachedTableFile::new(CLIENTS_PATH.to_owned()).expect("failed to create table_clients");
    for client in clients.iter_mut() {
        assert!(
            table_clients
                .insert(client)
                .expect("failed to insert client")
        );
    }

    if Path::new(ENTREPRISES_PATH).exists() {
        remove_dir_all(ENTREPRISES_PATH).expect("ENTREPRISES_PATH already exists");
    }
    let mut table_entreprises = CachedTableFile::new(ENTREPRISES_PATH.to_owned())
        .expect("failed to create table_entreprises");
    for entreprise in entreprises.iter_mut() {
        assert!(
            table_entreprises
                .insert(entreprise)
                .expect("failed to insert entreprises")
        );
    }

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
            match client.entreprise.data(&table_entreprises) {
                TableGet::Found(f) => f,
                e => panic!("get client.entreprise.data {e:?}"),
            }
            .nom,
            match table_entreprises.get_by_id(client.entreprise.id()) {
                TableGet::Found(f) => f,
                e => panic!("get table_entreprises.get_by_id {e:?}"),
            }
            .nom
        );
    }

    match table_clients.get_by_nom(&DynanicBinary::new(String::from("Will"))) {
        TableGet::Found(clients) => {
            assert_eq!(1, clients.len());
            assert_eq!(3, clients[0].id);
            assert_eq!("Will", clients[0].nom.data());
            assert_eq!(2, *clients[0].entreprise.id());
        }
        TableGet::NotFound => panic!("client Will suppossed to be found"),
        TableGet::InternalError(error) => panic!("{error}"),
        TableGet::Err(error) => panic!("{error:?}"),
    }
    assert!(
        table_clients
            .insert(&mut Client {
                id: 4,
                nom: DynanicBinary::new(String::from("Will")),
                entreprise: Foreign::new(2),
            })
            .expect("insert")
    );
    match table_clients.get_by_nom(&DynanicBinary::new(String::from("Will"))) {
        TableGet::Found(clients) => {
            assert_eq!(2, clients.len());
            assert_eq!(3, clients[0].id);
            assert_eq!("Will", clients[0].nom.data());
            assert_eq!(2, *clients[0].entreprise.id());

            assert_eq!(4, clients[1].id);
            assert_eq!("Will", clients[1].nom.data());
            assert_eq!(2, *clients[1].entreprise.id());
        }
        TableGet::NotFound => panic!("client Will suppossed to be found"),
        TableGet::InternalError(error) => panic!("{error}"),
        TableGet::Err(error) => panic!("{error:?}"),
    }
    table_clients.remove(&3).expect("remove");
    match table_clients.get_by_nom(&DynanicBinary::new(String::from("Will"))) {
        TableGet::Found(clients) => {
            assert_eq!(1, clients.len());
            assert_eq!(4, clients[0].id);
            assert_eq!("Will", clients[0].nom.data());
            assert_eq!(2, *clients[0].entreprise.id());
        }
        TableGet::NotFound => panic!("client Will suppossed to be found"),
        TableGet::InternalError(error) => panic!("{error}"),
        TableGet::Err(error) => panic!("{error:?}"),
    }
    match table_clients.get_by_nom(&DynanicBinary::new(String::from("NONE"))) {
        TableGet::Found(_) => panic!("client NONE not suppossed to be found"),
        TableGet::NotFound => {}
        TableGet::InternalError(error) => panic!("{error}"),
        TableGet::Err(error) => panic!("{error:?}"),
    }

    table_entreprises
        .clear()
        .expect("failed to clear entreprise");
    for client in table_clients.get_all().expect("client doesnt exists") {
        match client.entreprise.data(&table_entreprises) {
            TableGet::NotFound => {}
            e => panic!("not empty {e:?}"),
        }
    }
}
