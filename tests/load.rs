// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
/// We're using `#[test_log::test]` tests in this file which allows
/// you to see the log in your test runner if you set the environment
/// variable `RUST_LOG=info` (or debug or trace) and add `--nocapture`
/// at the end of your cargo test command line.
/// See https://crates.io/crates/test-log.
///
/// TODO: Add test for "import axioms" (add test ontology)
use std::{ops::Deref, path::Path};

use {
    indoc::formatdoc,
    iref::Iri,
    rdfox::{
        DataStore,
        DataStoreConnection,
        Error,
        FactDomain,
        GraphConnection,
        Parameters,
        Prefixes,
        RoleCreds,
        Server,
        ServerConnection,
        Statement,
        Transaction,
        APPLICATION_N_QUADS,
    },
    std::sync::Arc,
};

fn test_define_data_store() -> Result<Arc<DataStore>, Error> {
    log::info!("test_define_data_store");
    let data_store_params = Parameters::empty()?;
    DataStore::declare_with_parameters("example", data_store_params)
}

fn test_create_server() -> Result<Arc<Server>, Error> {
    log::info!("test_create_server");
    let server_params = &Parameters::empty()?
        .api_log(true)?
        .api_log_directory(Path::new("./tests"))?;
    Server::start_with_parameters(RoleCreds::default(), server_params)
}

fn test_create_server_connection(server: Arc<Server>) -> Result<Arc<ServerConnection>, Error> {
    log::info!("test_create_server_connection");

    let server_connection = server.connection_with_default_role()?;

    assert!(server_connection.get_number_of_threads()? > 0);

    // We next specify how many threads the server should use during import of
    // data and reasoning.
    server_connection.set_number_of_threads(2)?;

    assert_eq!(server_connection.get_number_of_threads()?, 2);

    Ok(server_connection)
}

fn test_create_data_store_connection(
    server_connection: &Arc<ServerConnection>,
    data_store: &Arc<DataStore>,
) -> Result<Arc<DataStoreConnection>, Error> {
    log::info!("test_create_data_store");

    server_connection.create_data_store(data_store)?;
    server_connection.connect_to_data_store(data_store)
}

fn test_create_graph(
    ds_connection: &Arc<DataStoreConnection>,
) -> Result<Arc<GraphConnection>, Error> {
    log::info!("test_create_graph");
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

    Ok(GraphConnection::new(
        ds_connection.clone(),
        test_graph,
        None,
    ))
}

#[allow(dead_code)]
fn test_count_some_stuff_in_the_store(
    tx: Arc<Transaction>,
    ds_connection: &Arc<DataStoreConnection>,
) -> Result<(), Error> {
    log::info!("test_count_some_stuff_in_the_store");
    let count = ds_connection.get_triples_count(tx, FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    Ok(())
}

#[allow(dead_code)]
fn test_count_some_stuff_in_the_graph(
    tx: Arc<Transaction>,
    graph_connection: &GraphConnection,
) -> Result<(), Error> {
    log::info!("test_count_some_stuff_in_the_graph");
    let count = graph_connection.get_triples_count(tx, FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    Ok(())
}

#[allow(dead_code)]
fn test_cursor_with_lexical_value(
    tx: Arc<Transaction>,
    graph_connection: &Arc<GraphConnection>,
) -> Result<(), Error> {
    log::info!("test_cursor_with_lexical_value");
    let graph = graph_connection.graph.as_display_iri();
    let prefixes = Prefixes::empty()?;
    let query = Statement::new(
        prefixes,
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
    let mut cursor = query.cursor(
        &graph_connection.data_store_connection,
        &Parameters::empty()?.fact_domain(FactDomain::ASSERTED)?,
        None,
    )?;

    let count = cursor.consume(tx, 10000, |row| {
        assert_eq!(row.opened.arity, 3);
        for term_index in 0..row.opened.arity {
            let value = row.lexical_value(term_index)?;
            log::info!("{value:?}");
        }
        Result::<(), Error>::Ok(())
    })?;
    log::info!("Number of rows processed: {count}");
    Ok(())
}

#[allow(dead_code)]
fn test_cursor_with_resource_value(
    tx: Arc<Transaction>,
    graph_connection: &Arc<GraphConnection>,
) -> Result<(), Error> {
    log::info!("test_cursor_with_resource_value");
    let graph = graph_connection.graph.as_display_iri();
    let prefixes = Prefixes::empty()?;
    let query = Statement::new(
        prefixes,
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
    let mut cursor = query.cursor(
        &graph_connection.data_store_connection,
        &Parameters::empty()?.fact_domain(FactDomain::ASSERTED)?,
        None,
    )?;

    let count = cursor.consume(tx, 10000, |row| {
        assert_eq!(row.opened.arity, 3);
        for term_index in 0..row.opened.arity {
            let value = row.resource_value(term_index)?;
            log::info!("{value:?}");
        }
        Result::<(), Error>::Ok(())
    })?;
    log::info!("Number of rows processed: {count}");
    Ok(())
}

#[allow(dead_code)]
fn test_run_query_to_nquads_buffer(
    _tx: Arc<Transaction>, // TODO: consider passing tx to evaluate_to_stream()
    ds_connection: &Arc<DataStoreConnection>,
) -> Result<(), Error> {
    log::info!("test_run_query_to_nquads_buffer");
    let prefixes = Prefixes::empty()?;
    let nquads_query = Statement::nquads_query(prefixes)?;
    let writer = std::io::stdout();
    ds_connection.evaluate_to_stream(
        writer,
        &nquads_query,
        APPLICATION_N_QUADS.deref(),
        None,
    )?;
    log::info!("test_run_query_to_nquads_buffer passed");
    Ok(())
}

#[test_log::test]
fn load_rdfox() -> Result<(), Error> {
    let server = test_create_server()?;
    let server_connection = test_create_server_connection(server)?;

    log::info!(
        "Server version is {}",
        server_connection.get_version()?
    );

    let data_store = test_define_data_store()?;

    // Create a separate scope to control the life-time of `ds_connection` which
    // will ensure that the DataStoreConnection created by
    // `test_create_data_store()` is destroyed at the end of this scope.
    {
        let ds_connection = test_create_data_store_connection(&server_connection, &data_store)?;

        let graph_connection = test_create_graph(&ds_connection)?;

        graph_connection.import_data_from_file("tests/test.ttl")?;

        Transaction::begin_read_only(&ds_connection)?.execute_and_rollback(|tx| {
            test_count_some_stuff_in_the_store(tx.clone(), &ds_connection)?;
            test_count_some_stuff_in_the_graph(tx.clone(), &graph_connection)?;

            test_cursor_with_lexical_value(tx.clone(), &graph_connection)?;
            test_cursor_with_resource_value(tx.clone(), &graph_connection)?;

            test_run_query_to_nquads_buffer(tx.clone(), &ds_connection)
        })?;
    }

    log::info!("Data store connection is now destroyed, now we can delete the data store:");

    server_connection.delete_data_store(&data_store)?;

    log::info!("load_rdfox end");

    Ok(())
}

#[test_log::test]
fn create_sandbox_database() -> Result<(), Error> {
    let server_params = Parameters::empty()?.switch_off_file_access_sandboxing()?;
    let role_creds = RoleCreds::default();
    let server = Server::start_with_parameters(role_creds, &server_params)?;

    let server_connection = server.connection_with_default_role()?;

    assert!(server_connection.get_number_of_threads()? > 0);

    // We next specify how many threads the server should use during import of
    // data and reasoning.
    server_connection.set_number_of_threads(2)
}
