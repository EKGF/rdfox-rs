// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::path::Path;
use env_logger::init;
use indoc::formatdoc;
use iref::Iri;
use rdfox::{FactDomain, GraphConnection, Parameters, Prefix, Prefixes, Statement};

/// TODO: Add test for "import axioms" (add test ontology)
#[test]
fn load_rdfox() -> Result<(), rdfox::Error> {
    init();
    let server_params = &Parameters::empty()?
        .api_log(true)?
        .api_log_directory(Path::new("./tests"))?;
    let server = rdfox::Server::start_with_parameters(
        rdfox::RoleCreds::default(),
        server_params
    )?;

    let connection = server.connection_with_default_role()?;

    assert!(connection.get_number_of_threads()? > 0);

    // We next specify how many threads the server should use during import of
    // data and reasoning.
    connection.set_number_of_threads(2)?;

    let data_store =
        connection.create_data_store(rdfox::DataStore::define("example"))?;

    let ds_connection = connection.connect_to_data_store(data_store)?;

    let graph_base_iri = Prefix::declare(
        "graph:",
        Iri::new("http://whatever.kom/graph/").unwrap(),
    );
    let test_graph = rdfox::Graph::declare(graph_base_iri, "test");

    assert_eq!(format!("{:}", test_graph).as_str(), "graph:test");
    assert_eq!(
        format!("{:}", test_graph.as_display_iri()).as_str(),
        "<http://whatever.kom/graph/test>"
    );

    let graph_connection =
        GraphConnection::new(&ds_connection, test_graph, None);
    graph_connection.import_data_from_file("tests/test.ttl")?;

    let count = ds_connection.get_triples_count(FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    let count = graph_connection.get_triples_count(FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    let graph = graph_connection.graph.as_display_iri();
    let prefixes = Prefixes::default()?;
    let query = Statement::query(
        &prefixes,
        formatdoc!(
            r##"
                SELECT ?subject ?predicate ?object
                FROM {graph}
                WHERE {{
                    ?subject a <https://ekgf.org/ontology/user-story/UserStory> ;
                        ?predicate ?object
                }}
                "##,
        )
        .as_str(),
    )?;
    let mut cursor = query.clone().cursor(
        graph_connection.data_store_connection,
        &Parameters::empty()?.fact_domain(FactDomain::ASSERTED)?,
    )?;

    let count = cursor.execute_and_rollback(|row| {
        assert_eq!(row.opened.arity, 3);
        for term_index in 0 .. row.opened.arity {

            let resource_id = row.resource_id(term_index)?;
            log::info!(
                "row={rowid} multiplicity={multiplicity} \
                 term_index={term_index} resource_id={resource_id}:",
                rowid = row.rowid,
                multiplicity = row.multiplicity
            );
            // let value = row.resource_value(resource_id)?;
            let value = row.resource_value_lexical_form(resource_id)?;
            log::info!("{value:?}");
            // log::info!("{}{}", value.prefix, value.value);
        }
        Ok(())
    })?;
    log::info!("Number of rows processed: {count}");

    let nquads_query = Statement::nquads_query(&prefixes)?;
    ds_connection.evaluate_to_buffer(nquads_query)?;


    Ok(())
}
