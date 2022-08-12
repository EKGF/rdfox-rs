// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ops::Deref;
use std::path::Path;
use env_logger::init;
use indoc::formatdoc;
use iref::Iri;

fn test_create_database() -> Result<rdfox::DataStoreConnection, rdfox::Error> {
    let server_params = &rdfox::Parameters::empty()?
        .api_log(true)?
        .api_log_directory(Path::new("./tests"))?;
    let server = rdfox::Server::start_with_parameters(
        rdfox::RoleCreds::default(),
        server_params,
    )?;

    let connection = server.connection_with_default_role()?;

    assert!(connection.get_number_of_threads()? > 0);

    // We next specify how many threads the server should use during import of
    // data and reasoning.
    connection.set_number_of_threads(2)?;

    assert_eq!(connection.get_number_of_threads()?, 2);

    let data_store = connection.create_data_store_named("example")?;

    connection.connect_to_data_store(data_store)
}

fn test_create_graph(
    ds_connection: &rdfox::DataStoreConnection
) -> Result<rdfox::GraphConnection, rdfox::Error> {
    let graph_base_iri = rdfox::Prefix::declare(
        "graph:",
        Iri::new("http://whatever.kom/graph/").unwrap(),
    );
    let test_graph = rdfox::Graph::declare(graph_base_iri, "test");

    assert_eq!(format!("{:}", test_graph).as_str(), "graph:test");
    assert_eq!(
        format!("{:}", test_graph.as_display_iri()).as_str(),
        "<http://whatever.kom/graph/test>"
    );

    Ok(rdfox::GraphConnection::new(
        ds_connection, test_graph, None,
    ))
}

#[allow(dead_code)]
fn test_count_some_stuff_in_the_store(ds_connection: &rdfox::DataStoreConnection) -> Result<(), rdfox::Error> {
    let count = ds_connection.get_triples_count(rdfox::FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    Ok(())
}

#[allow(dead_code)]
fn test_count_some_stuff_in_the_graph(graph_connection: &rdfox::GraphConnection) -> Result<(), rdfox::Error> {
    let count = graph_connection.get_triples_count(rdfox::FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    Ok(())
}

#[allow(dead_code)]
fn test_cursor_with_lexical_value(graph_connection: &rdfox::GraphConnection) -> Result<(), rdfox::Error> {
    let graph = graph_connection.graph.as_display_iri();
    let prefixes = rdfox::Prefixes::default()?;
    let query = rdfox::Statement::query(
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
        &rdfox::Parameters::empty()?.fact_domain(rdfox::FactDomain::ASSERTED)?,
    )?;

    let count = cursor.execute_and_rollback(|row| {
        assert_eq!(row.opened.arity, 3);
        for term_index in 0..row.opened.arity {
            let value = row.lexical_value(term_index)?;
            log::info!("{value:?}");
        }
        Ok(())
    })?;
    log::info!("Number of rows processed: {count}");
    Ok(())
}

#[allow(dead_code)]
fn test_cursor_with_resource_value(graph_connection: &rdfox::GraphConnection) -> Result<(), rdfox::Error> {
    let graph = graph_connection.graph.as_display_iri();
    let prefixes = rdfox::Prefixes::default()?;
    let query = rdfox::Statement::query(
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
        &rdfox::Parameters::empty()?.fact_domain(rdfox::FactDomain::ASSERTED)?,
    )?;

    let count = cursor.execute_and_rollback(|row| {
        assert_eq!(row.opened.arity, 3);
        for term_index in 0..row.opened.arity {
            let value = row.resource_value(term_index)?;
            log::info!("{value:?}");
        }
        Ok(())
    })?;
    log::info!("Number of rows processed: {count}");
    Ok(())
}

#[allow(dead_code)]
fn test_run_query_to_nquads_buffer(ds_connection: &rdfox::DataStoreConnection) -> Result<(), rdfox::Error> {
    let prefixes = rdfox::Prefixes::default()?;
    let nquads_query = rdfox::Statement::nquads_query(&prefixes)?;
    let writer= std::io::stdout();
    ds_connection.evaluate_to_stream(
        writer,
        &nquads_query,
        rdfox::APPLICATION_N_QUADS.deref(),
    )?;
    log::info!("test_run_query_to_nquads_buffer passed");
    Ok(())
}

/// TODO: Add test for "import axioms" (add test ontology)
#[test]
fn load_rdfox() -> Result<(), rdfox::Error> {
    init();

    let ds_connection = test_create_database()?;

    let graph_connection = test_create_graph(&ds_connection)?;

    graph_connection.import_data_from_file("tests/test.ttl")?;

    test_count_some_stuff_in_the_store(&ds_connection)?;
    test_count_some_stuff_in_the_graph(&graph_connection)?;

    test_cursor_with_lexical_value(&graph_connection)?;
    test_cursor_with_resource_value(&graph_connection)?;

    test_run_query_to_nquads_buffer(&ds_connection)?;

    log::info!("load_rdfox end");

    Ok(())
}
