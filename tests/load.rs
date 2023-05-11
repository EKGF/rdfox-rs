// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
// We're using `#[test_log::test]` tests in this file which allows
// you to see the log in your test runner if you set the environment
// variable `RUST_LOG=info` (or debug or trace) and add `--nocapture`
// at the end of your cargo test command line.
// See https://crates.io/crates/test-log.
//
// TODO: Add test for "import axioms" (add test ontology)
use {
    indoc::formatdoc,
    iref::Iri,
    rdf_store_rs::{
        consts::{APPLICATION_N_QUADS, PREFIX_SKOS},
        Graph,
        Literal,
        Prefix,
        RDFStoreError,
    },
    rdfox_rs::{
        DataStore,
        DataStoreConnection,
        FactDomain,
        GraphConnection,
        Parameters,
        PersistenceMode,
        Prefixes,
        RoleCreds,
        Server,
        ServerConnection,
        Statement,
        Transaction,
    },
    // std::path::Path,
    std::{ops::Deref, sync::Arc, thread::sleep, time::Duration},
};

fn test_define_data_store() -> Result<Arc<DataStore>, RDFStoreError> {
    tracing::info!("test_define_data_store");
    let data_store_params = Parameters::empty()?
        .persist_datastore(PersistenceMode::Off)?
        .persist_roles(PersistenceMode::Off)?;
    DataStore::declare_with_parameters("example", data_store_params)
}

fn test_create_server() -> Result<Arc<Server>, RDFStoreError> {
    tracing::info!("test_create_server");
    let server_params = Parameters::empty()?
        .persist_datastore(PersistenceMode::Off)?
        .persist_roles(PersistenceMode::Off)?;

    // TODO: The line below causes a SIGSEGV error when using the static link
    // library .api_log_directory(Path::new("./tests"))?;

    Server::start_with_parameters(RoleCreds::default(), Some(server_params))
}

fn test_create_server_connection(
    server: Arc<Server>,
) -> Result<Arc<ServerConnection>, RDFStoreError> {
    tracing::info!("test_create_server_connection");

    let server_connection = server.connection_with_default_role()?;

    let number_of_threads = server_connection.get_number_of_threads()?;
    tracing::info!("Using {number_of_threads} threads");

    let (max_used_bytes, available_bytes) = server_connection.get_memory_use()?;
    tracing::info!(
        "Memory use: max_used_bytes={max_used_bytes}, available_bytes={available_bytes}"
    );

    assert!(server_connection.get_number_of_threads()? > 0);

    // We next specify how many threads the server should use during import of
    // data and reasoning.
    server_connection.set_number_of_threads(2)?;

    assert_eq!(server_connection.get_number_of_threads()?, 2);

    Ok(server_connection)
}

#[allow(dead_code)]
fn test_create_data_store_connection(
    server_connection: &Arc<ServerConnection>,
    data_store: &Arc<DataStore>,
) -> Result<Arc<DataStoreConnection>, RDFStoreError> {
    tracing::info!("test_create_data_store");

    server_connection.create_data_store(data_store)?;
    server_connection.connect_to_data_store(data_store)
}

fn test_create_graph(
    ds_connection: &Arc<DataStoreConnection>,
    name: &str,
) -> Result<Arc<GraphConnection>, RDFStoreError> {
    tracing::info!("test_create_graph");
    let graph_base_iri = Prefix::declare(
        "graph:",
        Iri::new("https://whatever.kom/graph/").unwrap(),
    );
    let test_graph = Graph::declare(graph_base_iri, name);

    if name == "test" {
        assert_eq!(format!("{:}", test_graph).as_str(), "graph:test");
        assert_eq!(
            format!("{:}", test_graph.as_display_iri()).as_str(),
            "<https://whatever.kom/graph/test>"
        );
    }

    Ok(GraphConnection::new(
        ds_connection.clone(),
        test_graph,
        None,
    ))
}

#[allow(dead_code)]
fn test_count_some_stuff_in_the_store(
    tx: &Arc<Transaction>,
    ds_connection: &Arc<DataStoreConnection>,
) -> Result<(), RDFStoreError> {
    tracing::info!("test_count_some_stuff_in_the_store");
    let count = ds_connection.get_triples_count(tx, FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 1904);

    Ok(())
}

#[allow(dead_code)]
fn test_count_some_stuff_in_the_graph(
    tx: &Arc<Transaction>,
    graph_connection: &GraphConnection,
) -> Result<(), RDFStoreError> {
    tracing::info!("test_count_some_stuff_in_the_graph");
    let count = graph_connection.get_triples_count(tx, FactDomain::ALL);
    assert!(count.is_ok());
    assert_eq!(count.unwrap(), 37);

    Ok(())
}

#[allow(dead_code)]
fn test_cursor_with_lexical_value(
    tx: &Arc<Transaction>,
    graph_connection: &Arc<GraphConnection>,
) -> Result<(), RDFStoreError> {
    tracing::info!("test_cursor_with_lexical_value");
    let graph = graph_connection.graph.as_display_iri();
    let prefixes = Prefixes::empty()?;
    let query = Statement::new(
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
        .into(),
    )?;
    let mut cursor = query.cursor(
        &graph_connection.data_store_connection,
        &Parameters::empty()?.fact_domain(FactDomain::ASSERTED)?,
    )?;

    let count = cursor.consume(tx, 10000, |row| {
        assert_eq!(row.opened.arity, 3);
        for term_index in 0..row.opened.arity {
            let value = row.lexical_value(term_index)?;
            tracing::info!("{value:?}");
        }
        Result::<(), RDFStoreError>::Ok(())
    })?;
    tracing::info!("Number of rows processed: {count}");
    Ok(())
}

#[allow(dead_code)]
fn test_run_query_to_nquads_buffer(
    _tx: &Arc<Transaction>, // TODO: consider passing tx to evaluate_to_stream()
    ds_connection: &Arc<DataStoreConnection>,
) -> Result<(), RDFStoreError> {
    tracing::info!("test_run_query_to_nquads_buffer");
    let nquads_query = Statement::nquads_query(&Prefixes::empty()?)?;
    let writer = std::io::stdout();
    ds_connection.evaluate_to_stream(
        writer,
        &nquads_query,
        APPLICATION_N_QUADS.deref(),
        None,
    )?;
    tracing::info!("test_run_query_to_nquads_buffer passed");
    Ok(())
}

pub fn get_concept(
    concept_id: &Literal,
    graph_connection: &Arc<GraphConnection>,
) -> Result<Statement, RDFStoreError> {
    let prefix_concept = Prefix::declare_from_str("concept:", "https://ekgf.org/ontology/concept/");
    let prefixes = Prefixes::default()?
        .add_prefix(&prefix_concept)?
        .add_prefix(&PREFIX_SKOS)?;

    let graph = graph_connection.graph.as_display_iri();
    let sparql = formatdoc! {
        r##"
            SELECT DISTINCT ?key ?label ?comment ?data_type ?rdfs_class ?predicate
            WHERE {{
                VALUES ?concept {{
                    {concept_id}
                }}
                GRAPH {graph} {{
                    ?concept a concept:ClassConcept ; concept:key ?key .
                    OPTIONAL {{
                        ?concept rdfs:label ?label
                    }} .
                    OPTIONAL {{
                        ?concept rdfs:comment ?comment
                    }} .
                    OPTIONAL {{
                        ?concept concept:rdfsClass ?rdfs_class
                    }} .
                    OPTIONAL {{
                        ?concept concept:type ?data_type
                    }} .
                    OPTIONAL {{
                        ?concept concept:predicate ?predicate
                    }} .
                }}
            }}
            ORDER BY ?key
            "##
    };
    Ok(Statement::new(&prefixes, sparql.into())?)
}

#[allow(dead_code)]
fn test_query_concepts(
    tx: &Arc<Transaction>, // TODO: consider passing tx to evaluate_to_stream()
    graph_connection: &Arc<GraphConnection>,
) -> Result<(), RDFStoreError> {
    let concept_id = Literal::new_iri_reference_from_str(
        "https://placeholder.kg/id/concept-legal-person-legal-name-iri",
    )?;
    let statement = get_concept(&concept_id, graph_connection)?;
    let parameters = Parameters::empty()?.fact_domain(FactDomain::ALL)?;
    let mut cursor = statement.cursor(&tx.connection, &parameters)?;

    let count = cursor.consume(tx, 1000, |row| {
        tracing::info!("{row:?}");
        // for _term_index in 0..row.opened.arity {
        // if let Some(_value) = row.lexical_value(term_index)? {
        // } else {
        //     tracing::error!("{concept_id} is missing column
        // {term_index}:\n{statement:}"); }
        // }
        Ok::<(), RDFStoreError>(())
    })?;
    assert!(count > 0);

    Ok(())
}

#[test_log::test]
fn load_rdfox() -> Result<(), RDFStoreError> {
    eprintln!("running test load_rdfox:");
    tracing::info!("load_rdfox test start");
    let server = test_create_server()?;
    let server_connection = test_create_server_connection(server)?;

    tracing::info!(
        "Server version is {}",
        server_connection.get_version()?
    );

    let data_store = test_define_data_store()?;

    // Create a separate scope to control the life-time of `pool` which
    // will ensure that the DataStoreConnection created by
    // `pool_for()` is destroyed at the end of this scope.
    {
        let pool = data_store.pool_for(&server_connection, true, true)?;

        let conn = pool.get().unwrap();

        let graph_connection_test = test_create_graph(&conn, "test")?;
        let graph_connection_meta = test_create_graph(&conn, "meta")?;

        graph_connection_test.import_data_from_file("tests/test.ttl")?;
        graph_connection_meta.import_data_from_file("tests/concepts.ttl")?;

        Transaction::begin_read_only(&conn)?.execute_and_rollback(|ref tx| {
            test_count_some_stuff_in_the_store(tx, &conn)?;
            test_count_some_stuff_in_the_graph(tx, &graph_connection_test)?;
            test_cursor_with_lexical_value(tx, &graph_connection_test)?;
            test_run_query_to_nquads_buffer(tx, &conn)
        })?;
        Transaction::begin_read_only(&conn)?
            .execute_and_rollback(|ref tx| test_query_concepts(tx, &graph_connection_meta))?;
    }

    sleep(Duration::from_millis(500)); // wait for connection pool threads to end

    tracing::info!("Datastore connection is now destroyed, now we can delete the data store:");

    server_connection.delete_data_store(&data_store)?;

    tracing::info!("load_rdfox end");

    Ok(())
}
