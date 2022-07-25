// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use env_logger::init;
use iref::Iri;
use rdfox::{FactDomain, Prefix};

#[test]
fn load_rdfox() -> Result<(), rdfox::Error> {
    init();
    let server = rdfox::Server::start(rdfox::RoleCreds::default())?;

    let connection = server.connection_with_default_role()?;

    assert!(connection.get_number_of_threads()? > 0);

    // We next specify how many threads the server should use during import of data
    // and reasoning.
    connection.set_number_of_threads(2)?;

    let data_store = connection.create_data_store(rdfox::DataStore::define("example"))?;

    let ds_connection = connection.connect_to_data_store(data_store)?;

    let graph_base_iri =
        Prefix::declare("kggraph:", Iri::new("http://whatever.kom/graph/").unwrap());
    let test_graph = rdfox::Graph::declare(graph_base_iri, "test");
    ds_connection.import_data_from_file("test.ttl", &test_graph)?;

    let count = ds_connection.get_triples_count(FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 8);

    Ok(())
}
